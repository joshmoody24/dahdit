use crate::patterns::get_morse_pattern;
use crate::types::*;

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

/// Detected timing thresholds for morse interpretation
#[derive(Debug, Clone)]
struct MorseTimings {
    dot_duration: f32,
    dash_duration: f32,
    element_gap: f32,
    char_gap: f32,
    word_gap: f32,
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

        let off_durations: Vec<f32> = signals
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

        let dash_duration = if !dash_candidates.is_empty() {
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

        // Analyze OFF durations for gap detection
        let (element_gap, char_gap, word_gap) = if !off_durations.is_empty() {
            // Classify gaps based on prior expectations and observed dot duration
            let actual_element_gap = dot_duration;
            let actual_char_gap = dot_duration * 3.0;
            let actual_word_gap = dot_duration * 7.0;

            // Try to detect 3 distinct gap types using thresholds based on actual dot duration
            let short_gaps: Vec<f32> = off_durations
                .iter()
                .copied()
                .filter(|&d| d <= actual_element_gap * 2.0)
                .collect();

            let medium_gaps: Vec<f32> = off_durations
                .iter()
                .copied()
                .filter(|&d| d > actual_element_gap * 2.0 && d <= actual_char_gap * 1.5)
                .collect();

            let long_gaps: Vec<f32> = off_durations
                .iter()
                .copied()
                .filter(|&d| d > actual_char_gap * 1.5)
                .collect();

            let element_gap = if !short_gaps.is_empty() {
                TimingStats::new(short_gaps).unwrap().median
            } else {
                actual_element_gap
            };

            let char_gap = if !medium_gaps.is_empty() {
                TimingStats::new(medium_gaps).unwrap().median
            } else {
                actual_char_gap
            };

            let word_gap = if !long_gaps.is_empty() {
                TimingStats::new(long_gaps).unwrap().median
            } else {
                actual_word_gap
            };

            (element_gap, char_gap, word_gap)
        } else {
            // Fallback to standard morse ratios based on actual dot duration
            (dot_duration, dot_duration * 3.0, dot_duration * 7.0)
        };

        Ok(Self {
            dot_duration,
            dash_duration,
            element_gap,
            char_gap,
            word_gap,
        })
    }

    /// Classify an ON signal duration
    fn classify_element(&self, duration: f32) -> MorseElementType {
        let dot_diff = (duration - self.dot_duration).abs();
        let dash_diff = (duration - self.dash_duration).abs();

        if dot_diff <= dash_diff {
            MorseElementType::Dot
        } else {
            MorseElementType::Dash
        }
    }

    /// Classify an OFF signal duration
    fn classify_gap(&self, duration: f32) -> GapType {
        let element_diff = (duration - self.element_gap).abs();
        let char_diff = (duration - self.char_gap).abs();
        let word_diff = (duration - self.word_gap).abs();

        if element_diff <= char_diff && element_diff <= word_diff {
            GapType::IntraCharacter
        } else if char_diff <= word_diff {
            GapType::InterCharacter
        } else {
            GapType::Word
        }
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

/// Parse morse signals into text using state machine
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

    for signal in signals {
        if signal.seconds < NOISE_THRESHOLD {
            continue;
        }

        result.signals_processed += 1;

        match signal.on {
            true => {
                // ON signal - add element to current character
                let element = timings.classify_element(signal.seconds);
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
                // OFF signal - determine gap type
                let gap_type = timings.classify_gap(signal.seconds);
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
