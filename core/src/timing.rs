use crate::patterns::get_morse_pattern;
use crate::types::{MorseElement, MorseElementType, MorseTimingParams};
use std::time::{SystemTime, UNIX_EPOCH};

// ITU timing constants
const DOT_LENGTH_WPM: f32 = 1.2; // Standard ITU timing formula: dot duration = 1.2 / WPM seconds
const DOTS_PER_DASH: i32 = 3; // ITU specification: dash = 3 dot durations
const DOTS_PER_CHAR_GAP: i32 = 3; // ITU specification: inter-character gap = 3 dot durations
const DOTS_PER_WORD_GAP: i32 = 7; // ITU specification: inter-word gap = 7 dot durations
const HUMANIZATION_MAX_VARIANCE: f32 = 0.3; // Maximum timing variation as fraction of base duration

// Simple PRNG state for humanization - we need deterministic randomness
struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        // Use current time if seed is 0
        let actual_seed = if seed == 0 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u32
        } else {
            seed
        };
        Self {
            state: actual_seed.wrapping_add(1), // Ensure non-zero
        }
    }

    fn next_f32(&mut self) -> f32 {
        // Simple LCG (Linear Congruential Generator) - matches C rand() behavior roughly
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        // Normalize to [0, 1)
        (self.state >> 16) as f32 / 65536.0
    }
}

// Apply humanization - adds random variation to timing with bounded output
fn apply_humanization(
    base_duration: f32,
    humanization_factor: f32,
    rng: &mut Option<SimpleRng>,
) -> f32 {
    if humanization_factor <= 0.0 {
        return base_duration;
    }

    let variation = if let Some(rng) = rng {
        // Generate random variation: ±(humanization_factor * HUMANIZATION_MAX_VARIANCE) of base duration
        let max_variation = base_duration * humanization_factor * HUMANIZATION_MAX_VARIANCE;
        (rng.next_f32() - 0.5) * 2.0 * max_variation
    } else {
        0.0
    };

    let result = base_duration + variation;

    // Clamp result to safe bounds: [10% of base, base * (1 + max_variance)]
    let min_duration = base_duration * 0.1;
    let max_duration = base_duration * (1.0 + HUMANIZATION_MAX_VARIANCE);

    result.clamp(min_duration, max_duration)
}

/// Generate morse code timing elements from text
/// Returns the actual number of elements generated
pub fn morse_timing(text: &str, params: &MorseTimingParams) -> Result<Vec<MorseElement>, String> {
    if params.wpm <= 0 {
        return Err("Invalid WPM".to_string());
    }

    let mut rng = if params.humanization_factor > 0.0 {
        Some(SimpleRng::new(params.random_seed))
    } else {
        None
    };

    let dot_sec = DOT_LENGTH_WPM / params.wpm as f32;
    let mut elements = Vec::new();
    let mut chars = text.bytes().peekable();

    while let Some(ch) = chars.next() {
        // Handle spaces as inter-word gaps
        if ch == b' ' {
            let word_gap_duration = dot_sec * DOTS_PER_WORD_GAP as f32 * params.word_gap_multiplier;
            let duration =
                apply_humanization(word_gap_duration, params.humanization_factor, &mut rng);

            elements.push(MorseElement {
                element_type: MorseElementType::Gap,
                duration_seconds: duration,
            });
            continue;
        }

        // Handle prosigns in brackets [...]
        if ch == b'[' {
            let mut prosign_char_count = 0;

            // Process characters inside brackets (skip spaces and invalid chars)
            while let Some(&prosign_ch) = chars.peek() {
                if prosign_ch == b']' {
                    chars.next(); // consume the closing bracket
                    break;
                }

                let prosign_ch = chars.next().unwrap();

                // Skip spaces inside prosigns
                if prosign_ch == b' ' {
                    continue;
                }

                if let Some(pattern) = get_morse_pattern(prosign_ch) {
                    // Add 1-dot gap between characters in prosign (except for first character)
                    if prosign_char_count > 0 {
                        let duration =
                            apply_humanization(dot_sec, params.humanization_factor, &mut rng);
                        elements.push(MorseElement {
                            element_type: MorseElementType::Gap,
                            duration_seconds: duration,
                        });
                    }

                    // Add pattern elements
                    for (i, &element_type) in pattern.iter().enumerate() {
                        let base_duration = match element_type {
                            MorseElementType::Dot => dot_sec,
                            MorseElementType::Dash => dot_sec * DOTS_PER_DASH as f32,
                            MorseElementType::Gap => dot_sec, // shouldn't happen in patterns
                        };
                        let duration =
                            apply_humanization(base_duration, params.humanization_factor, &mut rng);

                        elements.push(MorseElement {
                            element_type,
                            duration_seconds: duration,
                        });

                        // Add inter-element gap (except after last element)
                        if i < pattern.len() - 1 {
                            let gap_duration =
                                apply_humanization(dot_sec, params.humanization_factor, &mut rng);
                            elements.push(MorseElement {
                                element_type: MorseElementType::Gap,
                                duration_seconds: gap_duration,
                            });
                        }
                    }
                    prosign_char_count += 1;
                }
            }
        } else {
            // Handle regular character
            if let Some(pattern) = get_morse_pattern(ch) {
                // Add inter-character gap if not the first character
                if !elements.is_empty() {
                    // Check if last element was not already a gap to avoid duplicate gaps
                    let should_add_gap = elements
                        .last()
                        .map(|e| e.element_type != MorseElementType::Gap)
                        .unwrap_or(true);

                    if should_add_gap {
                        let inter_char_duration = dot_sec * DOTS_PER_CHAR_GAP as f32;
                        let duration = apply_humanization(
                            inter_char_duration,
                            params.humanization_factor,
                            &mut rng,
                        );
                        elements.push(MorseElement {
                            element_type: MorseElementType::Gap,
                            duration_seconds: duration,
                        });
                    }
                }

                // Add pattern elements
                for (i, &element_type) in pattern.iter().enumerate() {
                    let base_duration = match element_type {
                        MorseElementType::Dot => dot_sec,
                        MorseElementType::Dash => dot_sec * DOTS_PER_DASH as f32,
                        MorseElementType::Gap => dot_sec, // shouldn't happen in patterns
                    };
                    let duration =
                        apply_humanization(base_duration, params.humanization_factor, &mut rng);

                    elements.push(MorseElement {
                        element_type,
                        duration_seconds: duration,
                    });

                    // Add inter-element gap (except after last element)
                    if i < pattern.len() - 1 {
                        let gap_duration =
                            apply_humanization(dot_sec, params.humanization_factor, &mut rng);
                        elements.push(MorseElement {
                            element_type: MorseElementType::Gap,
                            duration_seconds: gap_duration,
                        });
                    }
                }
            }
        }
    }

    Ok(elements)
}

/// Calculate size needed for timing elements (without actually generating them)
pub fn morse_timing_size(text: &str, params: &MorseTimingParams) -> Result<usize, String> {
    // For size calculation, we can just generate the actual elements and count them
    // This is simpler than duplicating the complex logic, and performance impact is minimal
    morse_timing(text, params).map(|elements| elements.len())
}
