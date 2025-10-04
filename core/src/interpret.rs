use crate::types::*;
use std::f32::consts::PI;

// === PHASE 3 CONSTANTS ===

// Beam Search Parameters - tuned for robustness with fuzzy signals
const DEFAULT_BEAM_SIZE: usize = 64; // Increased from 32 to keep more hypotheses
const DEFAULT_SPACE_PENALTY: f32 = 0.3; // Reduced from 0.5 - be less reluctant to add spaces
const DEFAULT_LM_WEIGHT: f32 = 2.0; // Increased from 1.0 - trust language model more
const DEFAULT_LATE_INTRA_PENALTY: f32 = 0.5; // Reduced from 0.7 - be more forgiving of timing
const DEFAULT_LONG_INTER_PENALTY: f32 = 0.6; // Reduced from 0.8 - be more forgiving of timing

// Language Model Parameters
const DEFAULT_UNKNOWN_TRIGRAM_COST: f32 = 8.0;

// Timing Thresholds (multipliers of unit time T)
const LATE_INTRA_THRESHOLD_MULTIPLIER: f32 = 2.0;
const LONG_INTER_THRESHOLD_MULTIPLIER: f32 = 4.0;
const LONG_WORD_LENGTH_THRESHOLD: u16 = 3;

// Probabilistic Timing Model Parameters - tuned for robustness with fuzzy signals
const DEFAULT_TIMING_SIGMA: f32 = 0.5; // Increased from 0.35 - less confident timing
const DEFAULT_TIMING_TRACKER_ALPHA: f32 = 0.1;

// Confidence Calculation Parameters

/// Timing statistics for adaptive analysis
#[derive(Debug, Clone)]
struct TimingStats {
    median: f32,
}

impl TimingStats {
    fn new(mut values: Vec<f32>) -> Option<Self> {
        if values.is_empty() {
            return None;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let len = values.len();

        let median = if len.is_multiple_of(2) {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        };

        Some(Self { median })
    }
}

// ===== PHASE 1: PROBABILISTIC TIMING MODEL =====

/// Log-normal probability density function
fn ln_pdf_lognormal(d: f32, mu: f32, sigma: f32) -> f32 {
    // mu and sigma are in log space; returns log-likelihood
    let x = d.max(1e-6); // Prevent log(0)
    let ln_x = x.ln();
    let z = (ln_x - mu) / sigma;
    let sqrt_2pi = (2.0 * PI).sqrt();
    -0.5 * z * z - ln_x - (sigma * sqrt_2pi).ln()
}

/// Online adaptive timing tracker using EWMA
#[derive(Debug, Clone)]
struct TimingTracker {
    ln_t: f32,  // Log of unit time (dot duration)
    alpha: f32, // EWMA smoothing factor (0.05 - 0.15)
}

impl TimingTracker {
    fn new(initial_t: f32) -> Self {
        Self {
            ln_t: initial_t.max(1e-6).ln(),
            alpha: DEFAULT_TIMING_TRACKER_ALPHA,
        }
    }

    /// Update timing estimate based on an ON signal
    fn update_from_on_signal(&mut self, duration: f32) {
        let ln_duration = duration.max(1e-6).ln();

        // Determine if this looks more like 1T or 3T
        let ln1t_diff = (ln_duration - self.ln_t).abs();
        let ln3t_diff = (ln_duration - (self.ln_t + 3.0f32.ln())).abs();

        let target_ln_t = if ln1t_diff < ln3t_diff {
            // Looks like a dot (1T)
            ln_duration
        } else {
            // Looks like a dash (3T) - so T = duration/3
            ln_duration - 3.0f32.ln()
        };

        // EWMA update
        self.ln_t = (1.0 - self.alpha) * self.ln_t + self.alpha * target_ln_t;
    }

    fn get_ln_t(&self) -> f32 {
        self.ln_t
    }

    /// Get the current unit time estimate
    fn get_t(&self) -> f32 {
        self.ln_t.exp()
    }
}

/// Probabilistic timing model using log-normal distributions with adaptive clustering
#[derive(Debug, Clone)]
struct ProbabilisticTimingModel {
    ln_t: f32,  // Log of unit time
    sigma: f32, // Log-space standard deviation
    /// Adaptive gap classification thresholds (learned from signal clustering)
    gap_clusters: GapClusters,
}

impl ProbabilisticTimingModel {
    fn from_tracker_and_clusters(tracker: &TimingTracker, gap_clusters: GapClusters) -> Self {
        Self {
            ln_t: tracker.get_ln_t(),
            sigma: DEFAULT_TIMING_SIGMA,
            gap_clusters,
        }
    }

    /// Get costs for classifying ON signals (negative log-likelihood)
    fn element_costs(&self, duration: f32) -> [(MorseElementType, f32); 2] {
        let ln_1t = self.ln_t;
        let ln_3t = self.ln_t + 3.0f32.ln();

        [
            (
                MorseElementType::Dot,
                -ln_pdf_lognormal(duration, ln_1t, self.sigma),
            ),
            (
                MorseElementType::Dash,
                -ln_pdf_lognormal(duration, ln_3t, self.sigma),
            ),
        ]
    }

    /// Get costs for classifying OFF signals using adaptive clustering thresholds
    fn gap_costs(&self, duration: f32) -> [(GapType, f32); 3] {
        // Use distance from cluster boundaries as costs
        // Closer to the "natural" boundary = lower cost

        let intra_cost = if duration <= self.gap_clusters.intra_to_inter_threshold {
            // Short gap - low cost for intra-character
            (duration - self.gap_clusters.intra_to_inter_threshold / 2.0).abs() * 0.1
        } else {
            // Not a short gap - higher cost for intra-character
            (duration - self.gap_clusters.intra_to_inter_threshold).abs() * 0.5
        };

        let inter_cost = if duration > self.gap_clusters.intra_to_inter_threshold
            && duration <= self.gap_clusters.inter_to_word_threshold
        {
            // Medium gap - low cost for inter-character
            let mid_point = (self.gap_clusters.intra_to_inter_threshold
                + self.gap_clusters.inter_to_word_threshold)
                / 2.0;
            (duration - mid_point).abs() * 0.1
        } else {
            // Not a medium gap - higher cost for inter-character
            let distance_from_range = if duration <= self.gap_clusters.intra_to_inter_threshold {
                self.gap_clusters.intra_to_inter_threshold - duration
            } else {
                duration - self.gap_clusters.inter_to_word_threshold
            };
            distance_from_range * 0.5
        };

        let word_cost = if duration > self.gap_clusters.inter_to_word_threshold {
            // Long gap - low cost for word gap
            (duration - self.gap_clusters.inter_to_word_threshold * 1.2).abs() * 0.1
        } else {
            // Not a long gap - higher cost for word gap
            (self.gap_clusters.inter_to_word_threshold - duration) * 0.5
        };

        [
            (GapType::IntraCharacter, intra_cost),
            (GapType::InterCharacter, inter_cost),
            (GapType::Word, word_cost),
        ]
    }

    /// Get minimum cost classification (for compatibility with existing FSM)
    fn classify_element_min_cost(&self, duration: f32) -> MorseElementType {
        let costs = self.element_costs(duration);
        if costs[0].1 <= costs[1].1 {
            costs[0].0
        } else {
            costs[1].0
        }
    }

    /// Get minimum cost gap classification (for compatibility with existing FSM)
    fn classify_gap_min_cost(&self, duration: f32) -> GapType {
        let costs = self.gap_costs(duration);
        let mut min_cost = costs[0].1;
        let mut min_type = costs[0].0;

        for &(gap_type, cost) in &costs[1..] {
            if cost < min_cost {
                min_cost = cost;
                min_type = gap_type;
            }
        }

        min_type
    }
}

// ===== PHASE 2: MORSE TRIE + LETTER COMPLETION =====

/// Morse trie node for efficient pattern matching
#[derive(Debug, Clone)]
struct TrieNode {
    /// Index to next node on dot (0 = no transition)
    next_dot: u16,
    /// Index to next node on dash (0 = no transition)
    next_dash: u16,
    /// Character if this is a terminal node (None = not terminal)
    terminal: Option<char>,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            next_dot: 0,
            next_dash: 0,
            terminal: None,
        }
    }
}

/// Morse trie for O(1) pattern-to-character lookup
struct MorseTrie {
    nodes: Vec<TrieNode>,
}

impl MorseTrie {
    /// Build the morse trie from all known patterns
    fn build() -> Self {
        let mut trie = Self {
            nodes: vec![TrieNode::new()], // Root node at index 0
        };

        // Add all characters, prioritizing uppercase for letters
        // First pass: add all non-letter characters
        for ch in 0u8..=255u8 {
            let char_val = ch as char;
            if let Some(pattern) = crate::patterns::get_morse_pattern(ch) {
                if !char_val.is_alphabetic() {
                    trie.add_pattern(pattern, char_val);
                }
            }
        }

        // Second pass: add uppercase letters (these will be the terminal characters)
        for ch in b'A'..=b'Z' {
            if let Some(pattern) = crate::patterns::get_morse_pattern(ch) {
                trie.add_pattern(pattern, ch as char);
            }
        }

        trie
    }

    /// Add a pattern to the trie
    fn add_pattern(&mut self, pattern: &[MorseElementType], terminal: char) {
        let mut current = 0usize; // Start at root

        for &element in pattern {
            let next_idx = match element {
                MorseElementType::Dot => {
                    if self.nodes[current].next_dot == 0 {
                        // Create new node
                        self.nodes.push(TrieNode::new());
                        let new_idx = self.nodes.len() - 1;
                        self.nodes[current].next_dot = new_idx as u16;
                        new_idx
                    } else {
                        self.nodes[current].next_dot as usize
                    }
                }
                MorseElementType::Dash => {
                    if self.nodes[current].next_dash == 0 {
                        // Create new node
                        self.nodes.push(TrieNode::new());
                        let new_idx = self.nodes.len() - 1;
                        self.nodes[current].next_dash = new_idx as u16;
                        new_idx
                    } else {
                        self.nodes[current].next_dash as usize
                    }
                }
                MorseElementType::Gap => {
                    // Gaps shouldn't appear in patterns
                    continue;
                }
            };
            current = next_idx;
        }

        // Mark the final node as terminal
        self.nodes[current].terminal = Some(terminal);
    }

    /// Get the next node index given current node and element
    fn transition(&self, node_idx: u16, element: MorseElementType) -> Option<u16> {
        if node_idx as usize >= self.nodes.len() {
            return None;
        }

        let node = &self.nodes[node_idx as usize];
        match element {
            MorseElementType::Dot => {
                if node.next_dot > 0 {
                    Some(node.next_dot)
                } else {
                    None
                }
            }
            MorseElementType::Dash => {
                if node.next_dash > 0 {
                    Some(node.next_dash)
                } else {
                    None
                }
            }
            MorseElementType::Gap => None, // Gaps don't cause trie transitions
        }
    }

    /// Check if a node is terminal and get its character
    fn get_terminal(&self, node_idx: u16) -> Option<char> {
        if node_idx as usize >= self.nodes.len() {
            return None;
        }
        self.nodes[node_idx as usize].terminal
    }

    /// Root node index
    const ROOT: u16 = 0;
}

/// Get the global morse trie (built on first call)
fn get_morse_trie() -> &'static MorseTrie {
    use std::sync::OnceLock;
    static TRIE: OnceLock<MorseTrie> = OnceLock::new();
    TRIE.get_or_init(MorseTrie::build)
}

// === PHASE 3: BEAM SEARCH + LANGUAGE MODEL ===

/// Beam search hypothesis for multiple interpretation paths
#[derive(Debug, Clone)]
struct Hypothesis {
    /// Current position in morse trie (0 = root)
    trie_node: u16,

    /// Last 2 characters for trigram language model context
    lm_context: [u8; 2],

    /// Number of characters in current word (since last space)
    pending_word_len: u16,

    /// Accumulated cost: timing + language + spacing penalties
    cost: f32,

    /// Decoded text output so far
    text: String,
}

impl Hypothesis {
    /// Create initial hypothesis at start of decoding
    fn new() -> Self {
        Self {
            trie_node: MorseTrie::ROOT,
            lm_context: [b' ', b' '], // Start with spaces for context
            pending_word_len: 0,
            cost: 0.0,
            text: String::new(),
        }
    }

    /// Add a character to the hypothesis and update language model context
    fn add_character(&mut self, ch: char, lm_cost: f32) {
        self.text.push(ch);
        self.cost += lm_cost;

        // Update trigram context (shift left, add new char)
        self.lm_context[0] = self.lm_context[1];
        self.lm_context[1] = if ch.is_ascii() { ch as u8 } else { b'?' };

        if ch == ' ' {
            self.pending_word_len = 0;
        } else {
            self.pending_word_len += 1;
        }

        // Reset trie position after completing a character
        self.trie_node = MorseTrie::ROOT;
    }

    /// Clone hypothesis for beam search expansion
    fn fork(&self) -> Self {
        self.clone()
    }
}

/// Simple character trigram language model with embedded English data
struct LanguageModel {
    /// Trigram costs: (char1, char2, char3) -> negative log probability
    trigrams: std::collections::HashMap<(u8, u8, u8), f32>,

    /// Default cost for unknown trigrams
    default_cost: f32,
}

impl LanguageModel {
    /// Create English language model with common trigrams
    /// Based on frequency analysis of English text
    fn new() -> Self {
        let mut lm = Self {
            trigrams: std::collections::HashMap::new(),
            default_cost: DEFAULT_UNKNOWN_TRIGRAM_COST,
        };

        // Load common English trigrams with frequency-based costs
        // Format: trigram, frequency_rank -> lower rank = lower cost
        lm.load_english_trigrams();

        // Add morse-specific patterns not in general English text
        lm.add_trigram_cost(b"SOS", 0.5); // Very common morse pattern
        lm.add_trigram_cost(b"CQC", 0.8); // Ham radio
        lm.add_trigram_cost(b"CQ ", 0.3); // CQ call
        lm.add_trigram_cost(b"QSO", 1.0); // Ham radio conversation

        lm
    }

    /// Load common English trigrams from build-time generated data
    fn load_english_trigrams(&mut self) {
        // Include the generated trigram data
        let trigram_data: &[(&str, f32)] = include!(concat!(env!("OUT_DIR"), "/trigrams.rs"));

        for &(trigram_str, cost) in trigram_data {
            let bytes = trigram_str.as_bytes();
            if bytes.len() == 3 {
                self.trigrams.insert((bytes[0], bytes[1], bytes[2]), cost);
            }
        }
    }

    /// Add a specific trigram with cost
    fn add_trigram_cost(&mut self, trigram: &[u8; 3], cost: f32) {
        self.trigrams
            .insert((trigram[0], trigram[1], trigram[2]), cost);
    }

    /// Get language model cost for completing a trigram
    fn get_cost(&self, context: [u8; 2], next_char: u8) -> f32 {
        self.trigrams
            .get(&(context[0], context[1], next_char))
            .copied()
            .unwrap_or(self.default_cost)
    }
}

/// Get the global language model (built on first call)
fn get_language_model() -> &'static LanguageModel {
    use std::sync::OnceLock;
    static LM: OnceLock<LanguageModel> = OnceLock::new();
    LM.get_or_init(LanguageModel::new)
}

/// Beam search parameters for Phase 3
#[derive(Debug, Clone)]
struct BeamSearchParams {
    /// Maximum number of hypotheses to maintain
    beam_size: usize,

    /// Cost penalty for inserting spaces
    space_penalty: f32,

    /// Weight of language model relative to timing costs
    lm_weight: f32,

    /// Penalty for late intra-character gaps (should be dots/dashes)
    late_intra_penalty: f32,

    /// Penalty for long inter-character gaps without inserting space
    long_inter_penalty: f32,
}

impl Default for BeamSearchParams {
    fn default() -> Self {
        Self {
            beam_size: DEFAULT_BEAM_SIZE,
            space_penalty: DEFAULT_SPACE_PENALTY,
            lm_weight: DEFAULT_LM_WEIGHT,
            late_intra_penalty: DEFAULT_LATE_INTRA_PENALTY,
            long_inter_penalty: DEFAULT_LONG_INTER_PENALTY,
        }
    }
}

/// Beam search decoder for morse signals using trie + language model
struct BeamSearchDecoder {
    /// Current hypotheses being tracked
    hypotheses: Vec<Hypothesis>,

    /// Search parameters
    params: BeamSearchParams,

    /// Access to morse trie
    trie: &'static MorseTrie,

    /// Access to language model
    lm: &'static LanguageModel,

    /// Timing tracker for online adaptation
    timing_tracker: TimingTracker,

    /// Probabilistic timing model for gap classification
    timing_model: ProbabilisticTimingModel,
}

impl BeamSearchDecoder {
    /// Create new beam search decoder with adaptive gap clustering
    fn new(timings: &MorseTimings, params: BeamSearchParams) -> Self {
        let timing_tracker = TimingTracker::new(timings.dot_duration);
        let timing_model = ProbabilisticTimingModel::from_tracker_and_clusters(
            &timing_tracker,
            timings.gap_clusters.clone(),
        );

        let mut decoder = Self {
            hypotheses: vec![Hypothesis::new()],
            params,
            trie: get_morse_trie(),
            lm: get_language_model(),
            timing_tracker,
            timing_model,
        };

        // Ensure we start with exactly one hypothesis
        decoder.hypotheses.truncate(1);
        decoder
    }

    /// Process an ON signal (dot or dash) - expand hypotheses in trie
    fn process_on_signal(&mut self, signal: &MorseSignal) {
        let element = self.timing_model.classify_element_min_cost(signal.seconds);
        let mut new_hypotheses = Vec::new();

        for hyp in &self.hypotheses {
            // Try to advance in trie with this element
            if let Some(next_node) = self.trie.transition(hyp.trie_node, element) {
                let mut new_hyp = hyp.fork();
                new_hyp.trie_node = next_node;
                // Get the cost for this specific element classification
                let element_costs = self.timing_model.element_costs(signal.seconds);
                for (elem_type, cost) in element_costs {
                    if elem_type == element {
                        new_hyp.cost += cost;
                        break;
                    }
                }
                new_hypotheses.push(new_hyp);
            }
            // Note: If trie transition fails, hypothesis is dropped (invalid pattern)
        }

        self.hypotheses = new_hypotheses;
        self.prune_beam();
    }

    /// Process an OFF signal (gap) - handle character/word completion and spacing
    fn process_off_signal(&mut self, signal: &MorseSignal) {
        let gap_type = self.timing_model.classify_gap_min_cost(signal.seconds);
        let mut new_hypotheses = Vec::new();

        for hyp in &self.hypotheses {
            match gap_type {
                GapType::IntraCharacter => {
                    // Short gap - stay in current character
                    let mut new_hyp = hyp.fork();
                    // Get the cost for this specific gap classification
                    let gap_costs = self.timing_model.gap_costs(signal.seconds);
                    for (gap_type_cost, cost) in gap_costs {
                        if gap_type_cost == gap_type {
                            new_hyp.cost += cost;
                            break;
                        }
                    }

                    // Add penalty if this looks like it should be longer
                    if signal.seconds
                        > self.timing_tracker.get_t() * LATE_INTRA_THRESHOLD_MULTIPLIER
                    {
                        new_hyp.cost += self.params.late_intra_penalty;
                    }

                    new_hypotheses.push(new_hyp);
                }
                GapType::InterCharacter => {
                    // Medium gap - complete character, don't add space
                    if let Some(ch) = self.trie.get_terminal(hyp.trie_node) {
                        let mut new_hyp = hyp.fork();
                        let lm_cost =
                            self.lm.get_cost(new_hyp.lm_context, ch as u8) * self.params.lm_weight;
                        new_hyp.add_character(ch, lm_cost);
                        // Get the cost for inter-character gap classification
                        let gap_costs = self.timing_model.gap_costs(signal.seconds);
                        for (gap_type_cost, cost) in gap_costs {
                            if gap_type_cost == gap_type {
                                new_hyp.cost += cost;
                                break;
                            }
                        }
                        new_hypotheses.push(new_hyp);
                    }

                    // Also consider adding space if word is getting long
                    if hyp.pending_word_len > LONG_WORD_LENGTH_THRESHOLD {
                        if let Some(ch) = self.trie.get_terminal(hyp.trie_node) {
                            let mut space_hyp = hyp.fork();
                            let ch_cost = self.lm.get_cost(space_hyp.lm_context, ch as u8)
                                * self.params.lm_weight;
                            space_hyp.add_character(ch, ch_cost);

                            let space_cost = self.lm.get_cost(space_hyp.lm_context, b' ')
                                * self.params.lm_weight;
                            space_hyp.add_character(' ', space_cost + self.params.space_penalty);
                            // Get the cost for gap classification with space
                            let gap_costs = self.timing_model.gap_costs(signal.seconds);
                            for (gap_type_cost, cost) in gap_costs {
                                if gap_type_cost == gap_type {
                                    space_hyp.cost += cost;
                                    break;
                                }
                            }
                            new_hypotheses.push(space_hyp);
                        }
                    }
                }
                GapType::Word => {
                    // Long gap - complete character and add space
                    if let Some(ch) = self.trie.get_terminal(hyp.trie_node) {
                        let mut new_hyp = hyp.fork();
                        let ch_cost =
                            self.lm.get_cost(new_hyp.lm_context, ch as u8) * self.params.lm_weight;
                        new_hyp.add_character(ch, ch_cost);

                        let space_cost =
                            self.lm.get_cost(new_hyp.lm_context, b' ') * self.params.lm_weight;
                        new_hyp.add_character(' ', space_cost);
                        // Get the cost for word gap classification
                        let gap_costs = self.timing_model.gap_costs(signal.seconds);
                        for (gap_type_cost, cost) in gap_costs {
                            if gap_type_cost == gap_type {
                                new_hyp.cost += cost;
                                break;
                            }
                        }
                        new_hypotheses.push(new_hyp);
                    }
                }
            }

            // For inter-character and word gaps, also try continuing without completing
            // (in case timing classification was wrong)
            if matches!(gap_type, GapType::InterCharacter) {
                let mut continue_hyp = hyp.fork();
                // Get the cost for treating this as intra-character gap
                let gap_costs = self.timing_model.gap_costs(signal.seconds);
                for (gap_type_cost, cost) in gap_costs {
                    if gap_type_cost == GapType::IntraCharacter {
                        continue_hyp.cost += cost;
                        break;
                    }
                }

                // Add penalty for long gaps without character completion
                if signal.seconds > self.timing_tracker.get_t() * LONG_INTER_THRESHOLD_MULTIPLIER {
                    continue_hyp.cost += self.params.long_inter_penalty;
                }

                new_hypotheses.push(continue_hyp);
            }
        }

        self.hypotheses = new_hypotheses;
        self.prune_beam();
    }

    /// Prune hypotheses to beam size, keeping lowest cost ones
    fn prune_beam(&mut self) {
        if self.hypotheses.len() <= self.params.beam_size {
            return;
        }

        // Sort by cost (ascending - lower is better)
        self.hypotheses.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep only the best beam_size hypotheses
        self.hypotheses.truncate(self.params.beam_size);
    }

    /// Complete decoding and return best hypothesis
    fn finalize(&mut self) -> Hypothesis {
        // Complete any remaining characters
        for hyp in &mut self.hypotheses {
            if let Some(ch) = self.trie.get_terminal(hyp.trie_node) {
                let lm_cost = self.lm.get_cost(hyp.lm_context, ch as u8) * self.params.lm_weight;
                hyp.add_character(ch, lm_cost);
            }
        }

        // Find hypothesis with lowest total cost
        std::mem::take(&mut self.hypotheses)
            .into_iter()
            .min_by(|a, b| {
                a.cost
                    .partial_cmp(&b.cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or_else(Hypothesis::new)
    }

    /// Update timing model with new signal (for online adaptation)
    fn update_timing(&mut self, signal: &MorseSignal) {
        if signal.on {
            self.timing_tracker.update_from_on_signal(signal.seconds);
            // Update probabilistic timing model with new tracker state but keep gap clusters
            self.timing_model = ProbabilisticTimingModel::from_tracker_and_clusters(
                &self.timing_tracker,
                self.timing_model.gap_clusters.clone(),
            );
        }
    }
}

/// Parse morse signals using beam search + language model (Phase 3)
fn parse_morse_signals_beam_search(
    signals: &[MorseSignal],
    timings: &MorseTimings,
    max_output_length: usize,
) -> MorseInterpretResult {
    let mut result = MorseInterpretResult {
        text: String::new(),
        confidence: 0.0,
        signals_processed: 0,
        patterns_recognized: 0,
    };

    if signals.is_empty() {
        return result;
    }

    // Initialize beam search with default parameters and adaptive gap clustering
    let params = BeamSearchParams::default();
    let mut decoder = BeamSearchDecoder::new(timings, params);

    let _recognized_patterns = 0;
    let _total_patterns = 0;

    // Process each signal through beam search
    for signal in signals {
        // Update timing model for adaptation
        decoder.update_timing(signal);

        // Process signal based on type
        match signal.on {
            true => {
                // ON signal - advance trie hypotheses
                decoder.process_on_signal(signal);
            }
            false => {
                // OFF signal - handle character/word completion
                decoder.process_off_signal(signal);
            }
        }

        // Safety check for output length
        if decoder
            .hypotheses
            .iter()
            .any(|h| h.text.len() >= max_output_length)
        {
            break;
        }

        result.signals_processed += 1;
    }

    // Finalize decoding and get best hypothesis
    let best_hypothesis = decoder.finalize();

    result.text = best_hypothesis.text;

    // Estimate confidence based on final cost and text length
    // Lower costs indicate higher confidence, but costs can be negative due to LM bonuses
    if !result.text.is_empty() {
        let avg_cost_per_char = best_hypothesis.cost / result.text.len() as f32;

        // Simple linear mapping based on observed cost ranges:
        // Negative costs (good English): confidence > 0.9
        // Cost 0-3: confidence 0.85-0.95
        // Cost 3-8: confidence 0.7-0.85
        // Cost > 8: confidence < 0.7
        result.confidence = if avg_cost_per_char < 0.0 {
            0.95 + (-avg_cost_per_char * 0.005).min(0.05) // Very high confidence for bonuses
        } else if avg_cost_per_char <= 3.0 {
            0.95 - avg_cost_per_char * 0.033 // 0.95 to 0.85
        } else if avg_cost_per_char <= 8.0 {
            0.85 - (avg_cost_per_char - 3.0) * 0.02 // 0.85 to 0.75
        } else {
            0.7 - (avg_cost_per_char - 8.0) * 0.01 // Decrease slowly below 0.7
        }
        .max(0.0);
    }

    // For beam search, we don't track individual patterns the same way
    // Instead, use text length as a proxy for recognized patterns
    result.patterns_recognized = result.text.chars().filter(|&c| c != ' ').count() as i32;

    result
}

/// Detected timing thresholds for morse interpretation using adaptive clustering
#[derive(Debug, Clone)]
struct MorseTimings {
    dot_duration: f32,
    /// Clustering-based gap thresholds (discovered from actual signal patterns)
    gap_clusters: GapClusters,
}

/// Gap classification thresholds discovered through clustering
#[derive(Debug, Clone)]
struct GapClusters {
    /// Threshold between intra-character and inter-character gaps
    intra_to_inter_threshold: f32,
    /// Threshold between inter-character and word gaps
    inter_to_word_threshold: f32,
}

impl MorseTimings {
    /// Create timings from signal analysis with adaptive clustering
    fn from_signals(signals: &[MorseSignal]) -> Result<Self, String> {
        // Hardcoded noise threshold - filter out very short signals
        const NOISE_THRESHOLD: f32 = 0.01;

        // Separate on and off signals, filtering noise
        let on_durations: Vec<f32> = signals
            .iter()
            .filter(|s| s.on && s.seconds >= NOISE_THRESHOLD)
            .map(|s| s.seconds)
            .collect();

        let off_durations: Vec<f32> = signals
            .iter()
            .filter(|s| !s.on && s.seconds >= NOISE_THRESHOLD)
            .map(|s| s.seconds)
            .collect();

        if on_durations.is_empty() {
            return Err("No valid on signals found".to_string());
        }

        // Analyze ON durations to find dot duration (use existing logic)
        let dot_duration = Self::find_dot_duration(&on_durations)?;

        // NEW: Cluster OFF durations to find natural gap boundaries
        let gap_clusters = Self::cluster_gap_durations(&off_durations, dot_duration)?;

        Ok(Self {
            dot_duration,
            gap_clusters,
        })
    }

    /// Find dot duration using existing dot/dash classification logic
    fn find_dot_duration(on_durations: &[f32]) -> Result<f32, String> {
        // Prior assumption about WPM for initial classification
        const PRIOR_WPM: i32 = 15;
        let expected_dot_duration = 1.2 / PRIOR_WPM as f32;
        let expected_dash_duration = expected_dot_duration * 3.0;

        let mut dot_candidates = Vec::new();
        let mut dash_candidates = Vec::new();

        // First pass: classify based on prior expectations
        for &duration in on_durations {
            let dot_diff = (duration - expected_dot_duration).abs();
            let dash_diff = (duration - expected_dash_duration).abs();

            if dot_diff <= dash_diff {
                dot_candidates.push(duration);
            } else {
                dash_candidates.push(duration);
            }
        }

        // Refine classification by finding natural breakpoint
        if !dot_candidates.is_empty() && !dash_candidates.is_empty() {
            let mut sorted_durations = on_durations.to_vec();
            sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            // Find the biggest gap between consecutive durations
            let mut best_split = (expected_dot_duration + expected_dash_duration) / 2.0;
            let mut max_gap = 0.0f32;

            for i in 0..sorted_durations.len() - 1 {
                let gap = sorted_durations[i + 1] - sorted_durations[i];
                if gap > max_gap {
                    max_gap = gap;
                    let potential_split = (sorted_durations[i] + sorted_durations[i + 1]) / 2.0;

                    if potential_split > expected_dot_duration * 0.5
                        && potential_split < expected_dash_duration * 1.5
                    {
                        best_split = potential_split;
                    }
                }
            }

            // Reclassify based on refined split point
            dot_candidates.clear();
            for &duration in on_durations {
                if duration <= best_split {
                    dot_candidates.push(duration);
                }
            }
        }

        if !dot_candidates.is_empty() {
            Ok(TimingStats::new(dot_candidates).unwrap().median)
        } else {
            Ok(expected_dot_duration) // fallback to prior
        }
    }

    /// Cluster OFF durations into short/medium/long gaps using adaptive thresholds
    fn cluster_gap_durations(
        off_durations: &[f32],
        dot_duration: f32,
    ) -> Result<GapClusters, String> {
        if off_durations.is_empty() {
            // No gaps - use traditional ratios as fallback
            return Ok(GapClusters {
                intra_to_inter_threshold: dot_duration * 2.0, // Between 1T and 3T
                inter_to_word_threshold: dot_duration * 5.0,  // Between 3T and 7T
            });
        }

        if off_durations.len() < 3 {
            // Too few gaps to cluster meaningfully - use generous thresholds
            let mut sorted_gaps = off_durations.to_vec();
            sorted_gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let mid_point = sorted_gaps[sorted_gaps.len() / 2];
            return Ok(GapClusters {
                intra_to_inter_threshold: mid_point * 0.7, // Generous threshold below median
                inter_to_word_threshold: mid_point * 1.5,  // Generous threshold above median
            });
        }

        // Sort gaps for clustering
        let mut sorted_gaps = off_durations.to_vec();
        sorted_gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Find natural breakpoints using largest gaps between consecutive values
        let mut gap_differences: Vec<(f32, usize)> = Vec::new();
        for i in 0..sorted_gaps.len() - 1 {
            let diff = sorted_gaps[i + 1] - sorted_gaps[i];
            gap_differences.push((diff, i));
        }

        // Sort by gap size (largest first)
        gap_differences.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Use the two largest gaps as cluster boundaries
        let mut breakpoints: Vec<usize> = gap_differences
            .iter()
            .take(2)
            .map(|(_, idx)| *idx)
            .collect();
        breakpoints.sort();

        let intra_to_inter_threshold = if !breakpoints.is_empty() {
            // Threshold between first and second cluster
            (sorted_gaps[breakpoints[0]] + sorted_gaps[breakpoints[0] + 1]) / 2.0
        } else {
            // Fallback: 1.5 * dot duration (between theoretical 1T and 3T)
            dot_duration * 1.5
        };

        let inter_to_word_threshold = if breakpoints.len() >= 2 {
            // Threshold between second and third cluster
            (sorted_gaps[breakpoints[1]] + sorted_gaps[breakpoints[1] + 1]) / 2.0
        } else {
            // Fallback: use a generous multiplier of the first threshold
            intra_to_inter_threshold * 2.5
        };

        Ok(GapClusters {
            intra_to_inter_threshold,
            inter_to_word_threshold,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GapType {
    IntraCharacter, // Intra-character gap (between dots/dashes within a character)
    InterCharacter, // Inter-character gap (between characters)
    Word,           // Inter-word gap
}

/// Main morse interpretation function
pub fn morse_interpret(
    signals: &[MorseSignal],
    params: &MorseInterpretParams,
) -> Result<MorseInterpretResult, String> {
    if signals.is_empty() {
        return Ok(MorseInterpretResult {
            text: String::new(),
            confidence: 0.0,
            signals_processed: 0,
            patterns_recognized: 0,
        });
    }

    // Analyze signal timings
    let timings = MorseTimings::from_signals(signals)?;

    // Parse signals into text using Phase 3 beam search + language model
    let result =
        parse_morse_signals_beam_search(signals, &timings, params.max_output_length as usize);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_signal(on: bool, seconds: f32) -> MorseSignal {
        MorseSignal { on, seconds }
    }

    #[test]
    fn test_empty_signals() {
        let params = MorseInterpretParams::default();
        let result = morse_interpret(&[], &params).unwrap();
        assert_eq!(result.text, "");
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_single_dot() {
        let params = MorseInterpretParams::default();
        // E = .
        let signals = vec![
            create_test_signal(true, 0.1),  // dot
            create_test_signal(false, 0.3), // character gap
        ];

        let result = morse_interpret(&signals, &params).unwrap();
        assert_eq!(result.text, "E");
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_hello() {
        let params = MorseInterpretParams::default();
        // H = ...., E = ., L = .-.., L = .-.., O = ---
        let dot = 0.1;
        let dash = 0.3;
        let element_gap = 0.1;
        let char_gap = 0.3;
        let _word_gap = 0.7;

        let signals = vec![
            // H = ....
            create_test_signal(true, dot),
            create_test_signal(false, element_gap),
            create_test_signal(true, dot),
            create_test_signal(false, element_gap),
            create_test_signal(true, dot),
            create_test_signal(false, element_gap),
            create_test_signal(true, dot),
            create_test_signal(false, char_gap),
            // E = .
            create_test_signal(true, dot),
            create_test_signal(false, char_gap),
            // L = .-..
            create_test_signal(true, dot),
            create_test_signal(false, element_gap),
            create_test_signal(true, dash),
            create_test_signal(false, element_gap),
            create_test_signal(true, dot),
            create_test_signal(false, element_gap),
            create_test_signal(true, dot),
            create_test_signal(false, char_gap),
            // L = .-..
            create_test_signal(true, dot),
            create_test_signal(false, element_gap),
            create_test_signal(true, dash),
            create_test_signal(false, element_gap),
            create_test_signal(true, dot),
            create_test_signal(false, element_gap),
            create_test_signal(true, dot),
            create_test_signal(false, char_gap),
            // O = ---
            create_test_signal(true, dash),
            create_test_signal(false, element_gap),
            create_test_signal(true, dash),
            create_test_signal(false, element_gap),
            create_test_signal(true, dash),
        ];

        let result = morse_interpret(&signals, &params).unwrap();
        assert_eq!(result.text, "HELLO");
        assert!(result.confidence > 0.8);
    }
}
