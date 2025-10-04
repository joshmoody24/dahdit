use crate::patterns::get_morse_pattern;
use crate::types::*;
use std::f32::consts::PI;

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
            alpha: 0.1, // Conservative smoothing
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
}

/// Probabilistic timing model using log-normal distributions
#[derive(Debug, Clone)]
struct ProbabilisticTimingModel {
    ln_t: f32,  // Log of unit time
    sigma: f32, // Log-space standard deviation
}

impl ProbabilisticTimingModel {
    fn from_tracker(tracker: &TimingTracker) -> Self {
        Self {
            ln_t: tracker.get_ln_t(),
            sigma: 0.35, // Reasonable default for human timing variation
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

    /// Get costs for classifying OFF signals (negative log-likelihood)
    fn gap_costs(&self, duration: f32) -> [(GapType, f32); 3] {
        let ln_1t = self.ln_t;
        let ln_3t = self.ln_t + 3.0f32.ln();
        let ln_7t = self.ln_t + 7.0f32.ln();

        [
            (
                GapType::IntraCharacter,
                -ln_pdf_lognormal(duration, ln_1t, self.sigma),
            ),
            (
                GapType::InterCharacter,
                -ln_pdf_lognormal(duration, ln_3t, self.sigma),
            ),
            (
                GapType::Word,
                -ln_pdf_lognormal(duration, ln_7t, self.sigma),
            ),
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

/// Detected timing thresholds for morse interpretation
/// Only dot_duration is used as initial estimate for TimingTracker
#[derive(Debug, Clone)]
struct MorseTimings {
    dot_duration: f32,
}

impl MorseTimings {
    /// Create timings from signal analysis with adaptive thresholds
    fn from_signals(signals: &[MorseSignal]) -> Result<Self, String> {
        // Hardcoded noise threshold - filter out very short signals
        const NOISE_THRESHOLD: f32 = 0.01;

        // Prior assumption about WPM - helps with initial classification
        // TODO: Consider parameterizing this in the future
        const PRIOR_WPM: i32 = 15;

        // Separate on and off signals, filtering noise
        let on_durations: Vec<f32> = signals
            .iter()
            .filter(|s| s.on && s.seconds >= NOISE_THRESHOLD)
            .map(|s| s.seconds)
            .collect();

        let _off_durations: Vec<f32> = signals
            .iter()
            .filter(|s| !s.on && s.seconds >= NOISE_THRESHOLD)
            .map(|s| s.seconds)
            .collect();

        if on_durations.is_empty() {
            return Err("No valid on signals found".to_string());
        }

        // Calculate expected dot duration from prior WPM assumption
        // At 15 WPM: dot = 1.2 / 15 = 0.08 seconds
        let expected_dot_duration = 1.2 / PRIOR_WPM as f32;
        let expected_dash_duration = expected_dot_duration * 3.0;

        // Analyze ON durations for dot/dash detection
        let _on_stats =
            TimingStats::new(on_durations.clone()).ok_or("Failed to analyze on signal timings")?;

        // Use prior knowledge to classify signals, then adapt based on observed data
        let mut dot_candidates = Vec::new();
        let mut dash_candidates = Vec::new();

        // First pass: classify based on prior expectations
        for &duration in &on_durations {
            let dot_diff = (duration - expected_dot_duration).abs();
            let dash_diff = (duration - expected_dash_duration).abs();

            if dot_diff <= dash_diff {
                dot_candidates.push(duration);
            } else {
                dash_candidates.push(duration);
            }
        }

        // If we have both types, look for a natural breakpoint to refine classification
        if !dot_candidates.is_empty() && !dash_candidates.is_empty() {
            let mut sorted_durations = on_durations.clone();
            sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            // Find the biggest gap between consecutive durations
            let mut best_split = (expected_dot_duration + expected_dash_duration) / 2.0;
            let mut max_gap = 0.0f32;

            for i in 0..sorted_durations.len() - 1 {
                let gap = sorted_durations[i + 1] - sorted_durations[i];
                if gap > max_gap {
                    max_gap = gap;
                    let potential_split = (sorted_durations[i] + sorted_durations[i + 1]) / 2.0;

                    // Only use this split if it's reasonable (between expected dot and dash)
                    if potential_split > expected_dot_duration * 0.5
                        && potential_split < expected_dash_duration * 1.5
                    {
                        best_split = potential_split;
                    }
                }
            }

            // Reclassify based on refined split point
            dot_candidates.clear();
            dash_candidates.clear();

            for &duration in &on_durations {
                if duration <= best_split {
                    dot_candidates.push(duration);
                } else {
                    dash_candidates.push(duration);
                }
            }
        }

        let dot_duration = if !dot_candidates.is_empty() {
            TimingStats::new(dot_candidates.clone()).unwrap().median
        } else {
            // No dots found - use expected duration or scale from dashes
            if !dash_candidates.is_empty() {
                let dash_median = TimingStats::new(dash_candidates.clone()).unwrap().median;
                dash_median / 3.0 // standard morse ratio
            } else {
                expected_dot_duration // fallback to prior
            }
        };

        let _dash_duration = if !dash_candidates.is_empty() {
            TimingStats::new(dash_candidates.clone()).unwrap().median
        } else {
            // No dashes found - use expected duration or scale from dots
            if !dot_candidates.is_empty() {
                dot_duration * 3.0 // standard morse ratio
            } else {
                expected_dash_duration // fallback to prior
            }
        };

        // Calculate expected gap durations from prior WPM assumption
        let _expected_element_gap = expected_dot_duration; // 1 dot duration
        let _expected_char_gap = expected_dot_duration * 3.0; // 3 dot durations
        let _expected_word_gap = expected_dot_duration * 7.0; // 7 dot durations

        Ok(Self { dot_duration })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GapType {
    IntraCharacter, // Intra-character gap (between dots/dashes within a character)
    InterCharacter, // Inter-character gap (between characters)
    Word,           // Inter-word gap
}

/// State machine for parsing morse signals
#[derive(Debug)]
enum ParseState {
    Idle,
    InCharacter(Vec<MorseElementType>),
    BetweenCharacters,
}

/// Parse morse signals into text using state machine with probabilistic timing
fn parse_morse_signals(
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

    let mut state = ParseState::Idle;
    let mut total_patterns = 0;
    let mut recognized_patterns = 0;

    const NOISE_THRESHOLD: f32 = 0.01;

    // Initialize adaptive timing tracker with initial estimate from timings
    let initial_t = timings.dot_duration;
    let mut timing_tracker = TimingTracker::new(initial_t);

    // Create probabilistic timing model (will be updated as we process signals)
    let mut prob_model = ProbabilisticTimingModel::from_tracker(&timing_tracker);

    for signal in signals {
        if signal.seconds < NOISE_THRESHOLD {
            continue;
        }

        result.signals_processed += 1;

        match signal.on {
            true => {
                // Update adaptive timing tracker with ON signal
                timing_tracker.update_from_on_signal(signal.seconds);

                // Update probabilistic model with latest timing estimate
                prob_model = ProbabilisticTimingModel::from_tracker(&timing_tracker);

                // ON signal - add element to current character using probabilistic classification
                let element = prob_model.classify_element_min_cost(signal.seconds);
                match state {
                    ParseState::Idle | ParseState::BetweenCharacters => {
                        state = ParseState::InCharacter(vec![element]);
                    }
                    ParseState::InCharacter(ref mut pattern) => {
                        pattern.push(element);

                        // Prevent patterns from getting too long
                        if pattern.len() > 7 {
                            // Force character completion for very long patterns
                            if let Some(ch) = pattern_to_character(pattern) {
                                result.text.push(ch);
                                recognized_patterns += 1;
                            }
                            total_patterns += 1;
                            state = ParseState::Idle;
                        }
                    }
                }
            }
            false => {
                // OFF signal - determine gap type using probabilistic classification
                let gap_type = prob_model.classify_gap_min_cost(signal.seconds);
                // Handle state transitions without borrowing conflicts
                let new_state = match &state {
                    ParseState::InCharacter(pattern) => {
                        match gap_type {
                            GapType::IntraCharacter => {
                                // Stay in character, continue building pattern
                                None // No state change
                            }
                            GapType::InterCharacter => {
                                // End of character
                                if let Some(ch) = pattern_to_character(pattern) {
                                    result.text.push(ch);
                                    recognized_patterns += 1;
                                }
                                total_patterns += 1;
                                Some(ParseState::BetweenCharacters)
                            }
                            GapType::Word => {
                                // End of character and word
                                if let Some(ch) = pattern_to_character(pattern) {
                                    result.text.push(ch);
                                    recognized_patterns += 1;
                                }
                                total_patterns += 1;
                                result.text.push(' ');
                                Some(ParseState::Idle)
                            }
                        }
                    }
                    _ => None, // Other states don't change on OFF signals
                };

                if let Some(new_state) = new_state {
                    state = new_state;
                }
            }
        }

        // Safety check for output length
        if result.text.len() >= max_output_length {
            break;
        }
    }

    // Handle any remaining pattern in final state
    if let ParseState::InCharacter(pattern) = state {
        if let Some(ch) = pattern_to_character(&pattern) {
            result.text.push(ch);
            recognized_patterns += 1;
        }
        total_patterns += 1;
    }

    result.patterns_recognized = recognized_patterns;

    // Calculate confidence based on recognition rate
    result.confidence = if total_patterns > 0 {
        (recognized_patterns as f32 / total_patterns as f32).min(1.0)
    } else {
        0.0
    };

    result
}

/// Convert a morse pattern to character using the existing lookup table
fn pattern_to_character(pattern: &[MorseElementType]) -> Option<char> {
    // Create reverse lookup map (pattern -> character)
    // This is inefficient for production but works for prototype
    for ch in 0u8..=255u8 {
        if let Some(stored_pattern) = get_morse_pattern(ch) {
            if stored_pattern.len() == pattern.len()
                && stored_pattern
                    .iter()
                    .zip(pattern.iter())
                    .all(|(a, b)| a == b)
            {
                return Some(ch as char);
            }
        }
    }
    None
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

    // Parse signals into text
    let result = parse_morse_signals(signals, &timings, params.max_output_length as usize);

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
