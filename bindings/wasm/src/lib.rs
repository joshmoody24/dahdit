// Clean WebAssembly bindings using pure serde for zero-duplication
use morse_core::{audio, interpret, timing, types::*};
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

// Combined configuration for both timing and audio parameters
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MorseConfig {
    // Timing parameters
    pub wpm: i32,
    pub word_gap_multiplier: f32,
    pub humanization_factor: f32,
    pub random_seed: u32,

    // Audio parameters
    pub sample_rate: i32,
    pub volume: f32,
    pub low_pass_cutoff: f32,
    pub high_pass_cutoff: f32,
    pub audio_mode: MorseAudioMode,

    // Radio mode parameters
    pub freq_hz: f32,
    pub waveform_type: MorseWaveformType,
    pub background_static_level: f32,

    // Telegraph mode parameters
    pub click_sharpness: f32,
    pub resonance_freq: f32,
    pub decay_rate: f32,
    pub mechanical_noise: f32,
    pub solenoid_response: f32,
    pub room_tone_level: f32,
    pub reverb_amount: f32,
}

impl Default for MorseConfig {
    fn default() -> Self {
        let timing_defaults = MorseTimingParams::default();
        let audio_defaults = MorseAudioParams::default();

        Self {
            // Timing defaults
            wpm: timing_defaults.wpm,
            word_gap_multiplier: timing_defaults.word_gap_multiplier,
            humanization_factor: timing_defaults.humanization_factor,
            random_seed: timing_defaults.random_seed,

            // Audio defaults
            sample_rate: audio_defaults.sample_rate,
            volume: audio_defaults.volume,
            low_pass_cutoff: audio_defaults.low_pass_cutoff,
            high_pass_cutoff: audio_defaults.high_pass_cutoff,
            audio_mode: audio_defaults.audio_mode,

            // Radio defaults
            freq_hz: audio_defaults.radio_params.freq_hz,
            waveform_type: audio_defaults.radio_params.waveform_type,
            background_static_level: audio_defaults.radio_params.background_static_level,

            // Telegraph defaults
            click_sharpness: audio_defaults.telegraph_params.click_sharpness,
            resonance_freq: audio_defaults.telegraph_params.resonance_freq,
            decay_rate: audio_defaults.telegraph_params.decay_rate,
            mechanical_noise: audio_defaults.telegraph_params.mechanical_noise,
            solenoid_response: audio_defaults.telegraph_params.solenoid_response,
            room_tone_level: audio_defaults.telegraph_params.room_tone_level,
            reverb_amount: audio_defaults.telegraph_params.reverb_amount,
        }
    }
}

impl MorseConfig {
    fn to_timing_params(&self) -> MorseTimingParams {
        MorseTimingParams {
            wpm: self.wpm,
            word_gap_multiplier: self.word_gap_multiplier,
            humanization_factor: self.humanization_factor,
            random_seed: self.random_seed,
        }
    }

    fn to_audio_params(&self) -> MorseAudioParams {
        MorseAudioParams {
            sample_rate: self.sample_rate,
            volume: self.volume,
            low_pass_cutoff: self.low_pass_cutoff,
            high_pass_cutoff: self.high_pass_cutoff,
            audio_mode: self.audio_mode,
            radio_params: MorseRadioParams {
                freq_hz: self.freq_hz,
                waveform_type: self.waveform_type,
                background_static_level: self.background_static_level,
            },
            telegraph_params: MorseTelegraphParams {
                click_sharpness: self.click_sharpness,
                resonance_freq: self.resonance_freq,
                decay_rate: self.decay_rate,
                mechanical_noise: self.mechanical_noise,
                solenoid_response: self.solenoid_response,
                room_tone_level: self.room_tone_level,
                reverb_amount: self.reverb_amount,
            },
        }
    }
}

// Pure serde-based API functions that return JSON strings

/// Generate morse timing elements as JSON
#[wasm_bindgen]
pub fn morse_timing_json(text: &str, config_json: &str) -> Result<String, JsValue> {
    let config: MorseConfig = if config_json.trim().is_empty() {
        MorseConfig::default()
    } else {
        serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid config JSON: {}", e)))?
    };

    let timing_params = config.to_timing_params();
    let elements = timing::morse_timing(text, &timing_params)
        .map_err(|e| JsValue::from_str(&e))?;

    serde_json::to_string(&elements)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

/// Generate morse audio as JSON (with embedded base64 audio data)
#[wasm_bindgen]
pub fn morse_audio_json(text: &str, config_json: &str) -> Result<String, JsValue> {
    let config: MorseConfig = if config_json.trim().is_empty() {
        MorseConfig::default()
    } else {
        serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid config JSON: {}", e)))?
    };

    // Generate timing elements
    let timing_params = config.to_timing_params();
    let timing_elements = timing::morse_timing(text, &timing_params)
        .map_err(|e| JsValue::from_str(&e))?;

    // Generate audio
    let audio_params = config.to_audio_params();
    let audio_data = audio::morse_audio(&timing_elements, &audio_params)
        .map_err(|e| JsValue::from_str(&e))?;

    // Calculate total duration
    let total_duration: f32 = timing_elements.iter().map(|e| e.duration_seconds).sum();

    // Return structured result as JSON
    let result = serde_json::json!({
        "audioData": audio_data,
        "sampleRate": audio_params.sample_rate,
        "duration": total_duration,
        "elements": timing_elements
    });

    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

/// Interpret morse signals from JSON
#[wasm_bindgen]
pub fn morse_interpret_json(signals_json: &str, config_json: &str) -> Result<String, JsValue> {
    let signals: Vec<MorseSignal> = serde_json::from_str(signals_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid signals JSON: {}", e)))?;

    let params: MorseInterpretParams = if config_json.trim().is_empty() {
        MorseInterpretParams::default()
    } else {
        serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid config JSON: {}", e)))?
    };

    let result = interpret::morse_interpret(&signals, &params)
        .map_err(|e| JsValue::from_str(&e))?;

    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

// Alternative API using wasm-bindgen's direct serde integration (experimental)

/// Generate morse timing using JsValue (direct serde integration)
#[wasm_bindgen]
pub fn morse_timing_direct(text: &str, config: &JsValue) -> Result<JsValue, JsValue> {
    let config: MorseConfig = if config.is_undefined() || config.is_null() {
        MorseConfig::default()
    } else {
        serde_wasm_bindgen::from_value(config.clone())?
    };

    let timing_params = config.to_timing_params();
    let elements = timing::morse_timing(text, &timing_params)
        .map_err(|e| JsValue::from_str(&e))?;

    serde_wasm_bindgen::to_value(&elements)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Generate morse audio using JsValue (direct serde integration)
#[wasm_bindgen]
pub fn morse_audio_direct(text: &str, config: &JsValue) -> Result<JsValue, JsValue> {
    let config: MorseConfig = if config.is_undefined() || config.is_null() {
        MorseConfig::default()
    } else {
        serde_wasm_bindgen::from_value(config.clone())?
    };

    // Generate timing and audio
    let timing_params = config.to_timing_params();
    let timing_elements = timing::morse_timing(text, &timing_params)
        .map_err(|e| JsValue::from_str(&e))?;

    let audio_params = config.to_audio_params();
    let audio_data = audio::morse_audio(&timing_elements, &audio_params)
        .map_err(|e| JsValue::from_str(&e))?;

    let total_duration: f32 = timing_elements.iter().map(|e| e.duration_seconds).sum();

    let result = serde_json::json!({
        "audioData": audio_data,
        "sampleRate": audio_params.sample_rate,
        "duration": total_duration,
        "elements": timing_elements
    });

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}