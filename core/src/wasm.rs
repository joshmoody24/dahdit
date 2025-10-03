// WebAssembly bindings to maintain JavaScript API compatibility
use crate::types::*;
use crate::{audio, timing};
use js_sys::Array;
use wasm_bindgen::prelude::*;

// Console logging for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// JavaScript-compatible result type
#[wasm_bindgen]
pub struct MorseTimingResult {
    elements: Vec<MorseElement>,
}

#[wasm_bindgen]
impl MorseTimingResult {
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.elements.len()
    }

    #[wasm_bindgen(getter)]
    pub fn elements(&self) -> Array {
        let array = Array::new();
        for element in &self.elements {
            let obj = js_sys::Object::new();
            let element_type = match element.element_type {
                MorseElementType::Dot => "dot",
                MorseElementType::Dash => "dash",
                MorseElementType::Gap => "gap",
            };
            js_sys::Reflect::set(&obj, &"type".into(), &element_type.into()).unwrap();
            js_sys::Reflect::set(
                &obj,
                &"duration_seconds".into(),
                &element.duration_seconds.into(),
            )
            .unwrap();
            array.push(&obj);
        }
        array
    }
}

#[wasm_bindgen]
pub struct MorseAudioResult {
    audio_data: Vec<f32>,
    sample_rate: i32,
    duration: f32,
}

#[wasm_bindgen]
impl MorseAudioResult {
    #[wasm_bindgen(getter)]
    pub fn audio_data(&self) -> Vec<f32> {
        self.audio_data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn sample_rate(&self) -> i32 {
        self.sample_rate
    }

    #[wasm_bindgen(getter)]
    pub fn duration(&self) -> f32 {
        self.duration
    }
}

// Main JavaScript API functions

#[wasm_bindgen]
pub fn generate_morse_timing(text: &str, config_json: &str) -> Result<MorseTimingResult, JsValue> {
    // Start with defaults, then overlay any provided config
    let params = if config_json.trim().is_empty() || config_json == "{}" {
        MorseTimingParams::default()
    } else {
        serde_json::from_str::<MorseTimingParams>(config_json)
            .unwrap_or_else(|_| MorseTimingParams::default())
    };

    timing::morse_timing(text, &params)
        .map(|elements| MorseTimingResult { elements })
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn generate_morse_audio(text: &str, config_json: &str) -> Result<MorseAudioResult, JsValue> {
    // Parse timing parameters using serde
    let timing_params = if config_json.trim().is_empty() || config_json == "{}" {
        MorseTimingParams::default()
    } else {
        serde_json::from_str::<MorseTimingParams>(config_json)
            .unwrap_or_else(|_| MorseTimingParams::default())
    };

    // Parse audio parameters using serde
    let audio_params = if config_json.trim().is_empty() || config_json == "{}" {
        MorseAudioParams::default()
    } else {
        serde_json::from_str::<MorseAudioParams>(config_json)
            .unwrap_or_else(|_| MorseAudioParams::default())
    };

    // Generate timing elements
    let timing_elements =
        timing::morse_timing(text, &timing_params).map_err(|e| JsValue::from_str(&e))?;

    // Calculate total duration
    let total_duration: f32 = timing_elements.iter().map(|e| e.duration_seconds).sum();

    // Generate audio
    let audio_data =
        audio::morse_audio(&timing_elements, &audio_params).map_err(|e| JsValue::from_str(&e))?;

    Ok(MorseAudioResult {
        audio_data,
        sample_rate: audio_params.sample_rate,
        duration: total_duration,
    })
}

#[wasm_bindgen]
pub struct MorseInterpretResultJs {
    text: String,
    confidence: f32,
    signals_processed: i32,
    patterns_recognized: i32,
}

#[wasm_bindgen]
impl MorseInterpretResultJs {
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    #[wasm_bindgen(getter)]
    pub fn signals_processed(&self) -> i32 {
        self.signals_processed
    }

    #[wasm_bindgen(getter)]
    pub fn patterns_recognized(&self) -> i32 {
        self.patterns_recognized
    }
}

#[wasm_bindgen]
pub fn interpret_morse_signals(
    signals_json: &str,
    config_json: &str,
) -> Result<MorseInterpretResultJs, JsValue> {
    // Parse signals from JSON
    let signals: Vec<crate::types::MorseSignal> = serde_json::from_str(signals_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid signals JSON: {}", e)))?;

    // Parse config from JSON, with defaults
    let params = if config_json.trim().is_empty() || config_json == "{}" {
        crate::types::MorseInterpretParams::default()
    } else {
        serde_json::from_str::<crate::types::MorseInterpretParams>(config_json)
            .unwrap_or_else(|_| crate::types::MorseInterpretParams::default())
    };

    // Use the morse interpret function from our interpret module
    let result =
        crate::interpret::morse_interpret(&signals, &params).map_err(|e| JsValue::from_str(&e))?;

    Ok(MorseInterpretResultJs {
        text: result.text,
        confidence: result.confidence,
        signals_processed: result.signals_processed,
        patterns_recognized: result.patterns_recognized,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_params_json_deserialization() {
        // Test with empty JSON - should use defaults
        let json = "{}";
        let params: MorseAudioParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.sample_rate, 44100);
        assert_eq!(params.volume, 0.5);
        assert_eq!(params.audio_mode, MorseAudioMode::Radio);

        // Test with partial JSON - should merge with defaults
        let json = r#"{"volume": 0.8, "sampleRate": 48000}"#;
        let params: MorseAudioParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.sample_rate, 48000);
        assert_eq!(params.volume, 0.8);
        assert_eq!(params.audio_mode, MorseAudioMode::Radio); // default

        // Test with radio mode parameters
        let json = r#"{"audioMode": 0, "freqHz": 880, "waveformType": 1}"#;
        let params: MorseAudioParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.audio_mode, MorseAudioMode::Radio);
        assert_eq!(params.radio_params.freq_hz, 880.0);
        assert_eq!(params.radio_params.waveform_type, MorseWaveformType::Square);

        // Test with telegraph mode parameters
        let json = r#"{"audioMode": 1, "clickSharpness": 0.7, "resonanceFreq": 1000}"#;
        let params: MorseAudioParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.audio_mode, MorseAudioMode::Telegraph);
        assert_eq!(params.telegraph_params.click_sharpness, 0.7);
        assert_eq!(params.telegraph_params.resonance_freq, 1000.0);
    }

    #[test]
    fn test_timing_params_json_deserialization() {
        // Test with empty JSON - should use defaults
        let json = "{}";
        let params: MorseTimingParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.wpm, 20);
        assert_eq!(params.word_gap_multiplier, 1.0);

        // Test with partial JSON
        let json = r#"{"wpm": 30, "humanizationFactor": 0.2}"#;
        let params: MorseTimingParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.wpm, 30);
        assert_eq!(params.humanization_factor, 0.2);
        assert_eq!(params.word_gap_multiplier, 1.0); // default
    }
}
