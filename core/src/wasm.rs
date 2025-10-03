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
pub fn generate_morse_timing_js(
    text: &str,
    wpm: i32,
    word_gap_multiplier: f32,
    humanization_factor: f32,
    random_seed: u32,
) -> Result<MorseTimingResult, JsValue> {
    let params = MorseTimingParams {
        wpm,
        word_gap_multiplier,
        humanization_factor,
        random_seed,
    };

    timing::morse_timing(text, &params)
        .map(|elements| MorseTimingResult { elements })
        .map_err(|e| JsValue::from_str(&e))
}

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
#[allow(clippy::too_many_arguments)]
pub fn generate_morse_audio_js(
    text: &str,
    // Timing parameters
    wpm: i32,
    word_gap_multiplier: f32,
    humanization_factor: f32,
    random_seed: u32,
    // Audio parameters
    sample_rate: i32,
    volume: f32,
    low_pass_cutoff: f32,
    high_pass_cutoff: f32,
    audio_mode: i32,
    // Radio parameters
    frequency: f32,
    waveform_type: i32,
    background_static_level: f32,
    // Telegraph parameters
    click_sharpness: f32,
    resonance_freq: f32,
    decay_rate: f32,
    mechanical_noise: f32,
    solenoid_response: f32,
    room_tone_level: f32,
    reverb_amount: f32,
) -> Result<MorseAudioResult, JsValue> {
    let timing_params = MorseTimingParams {
        wpm,
        word_gap_multiplier,
        humanization_factor,
        random_seed,
    };

    let audio_mode_enum = match audio_mode {
        0 => MorseAudioMode::Radio,
        1 => MorseAudioMode::Telegraph,
        _ => return Err(JsValue::from_str("Invalid audio mode")),
    };

    let waveform_type_enum = match waveform_type {
        0 => MorseWaveformType::Sine,
        1 => MorseWaveformType::Square,
        2 => MorseWaveformType::Sawtooth,
        3 => MorseWaveformType::Triangle,
        _ => return Err(JsValue::from_str("Invalid waveform type")),
    };

    let audio_params = MorseAudioParams {
        sample_rate,
        volume,
        low_pass_cutoff,
        high_pass_cutoff,
        audio_mode: audio_mode_enum,
        radio_params: MorseRadioParams {
            freq_hz: frequency,
            waveform_type: waveform_type_enum,
            background_static_level,
        },
        telegraph_params: MorseTelegraphParams {
            click_sharpness,
            resonance_freq,
            decay_rate,
            mechanical_noise,
            solenoid_response,
            room_tone_level,
            reverb_amount,
        },
    };

    // Generate timing elements
    let elements = timing::morse_timing(text, &timing_params).map_err(|e| JsValue::from_str(&e))?;

    // Calculate total duration
    let total_duration: f32 = elements.iter().map(|e| e.duration_seconds).sum();

    // Generate audio
    let audio_data =
        audio::morse_audio(&elements, &audio_params).map_err(|e| JsValue::from_str(&e))?;

    Ok(MorseAudioResult {
        audio_data,
        sample_rate,
        duration: total_duration,
    })
}

#[wasm_bindgen]
pub fn generate_morse_audio(text: &str, config_json: &str) -> Result<MorseAudioResult, JsValue> {
    // Parse JSON into a generic Value first
    let config_value: serde_json::Value = if config_json.trim().is_empty() || config_json == "{}" {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(config_json)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()))
    };

    // Extract timing and audio parameters using serde
    let timing_params: MorseTimingParams = serde_json::from_value(config_value.clone())
        .unwrap_or_else(|_| MorseTimingParams::default());

    let mut audio_params: MorseAudioParams = serde_json::from_value(config_value.clone())
        .unwrap_or_else(|_| MorseAudioParams::default());

    // Handle nested radio/telegraph params with camelCase field names
    if let Ok(radio_params) = serde_json::from_value::<MorseRadioParams>(config_value.clone()) {
        audio_params.radio_params = radio_params;
    }
    if let Ok(telegraph_params) = serde_json::from_value::<MorseTelegraphParams>(config_value) {
        audio_params.telegraph_params = telegraph_params;
    }

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

// Size calculation functions (for memory pre-allocation)
#[wasm_bindgen]
pub fn morse_timing_size_js(
    text: &str,
    wpm: i32,
    word_gap_multiplier: f32,
    humanization_factor: f32,
    random_seed: u32,
) -> Result<usize, JsValue> {
    let params = MorseTimingParams {
        wpm,
        word_gap_multiplier,
        humanization_factor,
        random_seed,
    };

    timing::morse_timing_size(text, &params).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn morse_audio_size_js(timing_elements_js: &Array, sample_rate: i32) -> Result<usize, JsValue> {
    // Convert JS array to Rust elements
    let mut elements = Vec::new();
    for i in 0..timing_elements_js.length() {
        let element_obj = timing_elements_js.get(i);
        let type_str = js_sys::Reflect::get(&element_obj, &"type".into())
            .map_err(|_| JsValue::from_str("Invalid element format"))?
            .as_string()
            .ok_or_else(|| JsValue::from_str("Element type must be string"))?;

        let duration = js_sys::Reflect::get(&element_obj, &"duration_seconds".into())
            .map_err(|_| JsValue::from_str("Invalid element format"))?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Duration must be number"))?
            as f32;

        let element_type = match type_str.as_str() {
            "dot" => MorseElementType::Dot,
            "dash" => MorseElementType::Dash,
            "gap" => MorseElementType::Gap,
            _ => return Err(JsValue::from_str("Invalid element type")),
        };

        elements.push(MorseElement {
            element_type,
            duration_seconds: duration,
        });
    }

    let params = MorseAudioParams {
        sample_rate,
        ..Default::default()
    };

    audio::morse_audio_size(&elements, &params).map_err(|e| JsValue::from_str(&e))
}

// Stub for interpreter (TODO as requested)
#[wasm_bindgen]
pub fn interpret_morse_signals_js(
    _signals_js: &Array,
    _max_k_means_iterations: i32,
    _convergence_threshold: f32,
    _noise_threshold: f32,
    _max_output_length: i32,
) -> Result<JsValue, JsValue> {
    // TODO: Implement morse interpretation
    Err(JsValue::from_str(
        "Morse interpretation not implemented yet",
    ))
}
