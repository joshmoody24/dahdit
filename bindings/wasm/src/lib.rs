// WebAssembly bindings to maintain JavaScript API compatibility
use morse_core::types::*;
use morse_core::{audio, timing};
use js_sys::Array;
use wasm_bindgen::prelude::*;

mod support;

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

// Macro to generate wasm_bindgen wrapper enums that mirror core enums
macro_rules! wasm_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $value:expr),* $(,)?
        }
        from $core_type:ty
    ) => {
        #[wasm_bindgen]
        $(#[$meta])*
        $vis enum $name {
            $($variant = $value),*
        }

        impl From<$core_type> for $name {
            fn from(value: $core_type) -> Self {
                match value {
                    $(<$core_type>::$variant => $name::$variant),*
                }
            }
        }

        impl From<$name> for $core_type {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$variant => <$core_type>::$variant),*
                }
            }
        }
    };
}

// Re-export enums with wasm_bindgen for JavaScript compatibility
wasm_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MorseAudioMode {
        Radio = 0,
        Telegraph = 1,
    }
    from morse_core::types::MorseAudioMode
}

wasm_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MorseWaveformType {
        Sine = 0,
        Square = 1,
        Sawtooth = 2,
        Triangle = 3,
    }
    from morse_core::types::MorseWaveformType
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
    let params = support::parse_with_defaults::<MorseTimingParams>(config_json);
    
    timing::morse_timing(text, &params)
        .map(|elements| MorseTimingResult { elements })
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn generate_morse_audio(text: &str, config_json: &str) -> Result<MorseAudioResult, JsValue> {
    // Parse JSON into a generic Value first for manual discriminated union handling
    let config_value: serde_json::Value = if config_json.trim().is_empty() || config_json == "{}" {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(config_json)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()))
    };

    // Debug: log the raw JSON received
    log(&format!("WASM RAW JSON: {}", config_json));

    // Extract timing parameters using our helper
    let timing_params: MorseTimingParams = serde_json::from_value(config_value.clone())
        .unwrap_or_else(|_| MorseTimingParams::default());

    // Parse audio parameters manually for discriminated union support
    let mut audio_params = MorseAudioParams::default();

    // Parse common parameters
    if let Some(volume) = config_value.get("volume") {
        if let Some(vol) = volume.as_f64() {
            audio_params.volume = vol as f32;
        }
    }
    if let Some(sample_rate) = config_value.get("sampleRate") {
        if let Some(sr) = sample_rate.as_i64() {
            audio_params.sample_rate = sr as i32;
        }
    }
    if let Some(audio_mode) = config_value.get("audioMode") {
        if let Some(mode) = audio_mode.as_i64() {
            audio_params.audio_mode = if mode == 1 {
                morse_core::types::MorseAudioMode::Telegraph
            } else {
                morse_core::types::MorseAudioMode::Radio
            };
        }
    }

    // Parse mode-specific parameters based on audioMode
    match audio_params.audio_mode {
        morse_core::types::MorseAudioMode::Radio => {
            if let Some(freq_hz) = config_value.get("freqHz") {
                if let Some(freq) = freq_hz.as_f64() {
                    audio_params.radio_params.freq_hz = freq as f32;
                }
            }
            if let Some(waveform_type) = config_value.get("waveformType") {
                if let Some(wt) = waveform_type.as_i64() {
                    audio_params.radio_params.waveform_type = match wt {
                        1 => morse_core::types::MorseWaveformType::Square,
                        2 => morse_core::types::MorseWaveformType::Sawtooth,
                        3 => morse_core::types::MorseWaveformType::Triangle,
                        _ => morse_core::types::MorseWaveformType::Sine,
                    };
                }
            }
            if let Some(bg_static) = config_value.get("backgroundStaticLevel") {
                if let Some(bg) = bg_static.as_f64() {
                    audio_params.radio_params.background_static_level = bg as f32;
                }
            }
        }
        morse_core::types::MorseAudioMode::Telegraph => {
            if let Some(click_sharpness) = config_value.get("clickSharpness") {
                if let Some(cs) = click_sharpness.as_f64() {
                    audio_params.telegraph_params.click_sharpness = cs as f32;
                }
            }
            if let Some(resonance_freq) = config_value.get("resonanceFreq") {
                if let Some(rf) = resonance_freq.as_f64() {
                    audio_params.telegraph_params.resonance_freq = rf as f32;
                }
            }
            if let Some(decay_rate) = config_value.get("decayRate") {
                if let Some(dr) = decay_rate.as_f64() {
                    audio_params.telegraph_params.decay_rate = dr as f32;
                }
            }
            if let Some(mechanical_noise) = config_value.get("mechanicalNoise") {
                if let Some(mn) = mechanical_noise.as_f64() {
                    audio_params.telegraph_params.mechanical_noise = mn as f32;
                }
            }
            if let Some(solenoid_response) = config_value.get("solenoidResponse") {
                if let Some(sr) = solenoid_response.as_f64() {
                    audio_params.telegraph_params.solenoid_response = sr as f32;
                }
            }
            if let Some(room_tone_level) = config_value.get("roomToneLevel") {
                if let Some(rtl) = room_tone_level.as_f64() {
                    audio_params.telegraph_params.room_tone_level = rtl as f32;
                }
            }
            if let Some(reverb_amount) = config_value.get("reverbAmount") {
                if let Some(ra) = reverb_amount.as_f64() {
                    audio_params.telegraph_params.reverb_amount = ra as f32;
                }
            }
        }
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
    let signals: Vec<morse_core::types::MorseSignal> = serde_json::from_str(signals_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid signals JSON: {}", e)))?;

    // Parse config with defaults
    let params = support::parse_with_defaults::<morse_core::types::MorseInterpretParams>(config_json);

    // Use the morse interpret function from our interpret module
    let result =
        morse_core::interpret::morse_interpret(&signals, &params).map_err(|e| JsValue::from_str(&e))?;

    Ok(MorseInterpretResultJs {
        text: result.text,
        confidence: result.confidence,
        signals_processed: result.signals_processed,
        patterns_recognized: result.patterns_recognized,
    })
}
