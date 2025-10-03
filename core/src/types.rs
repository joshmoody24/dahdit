use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorseElementType {
    Dot = 0,
    Dash = 1,
    Gap = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorseElement {
    pub element_type: MorseElementType,
    pub duration_seconds: f32,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MorseAudioMode {
    Radio = 0,
    Telegraph = 1,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MorseWaveformType {
    Sine = 0,
    Square = 1,
    Sawtooth = 2,
    Triangle = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MorseTimingParams {
    pub wpm: i32,
    pub word_gap_multiplier: f32,
    pub humanization_factor: f32,
    pub random_seed: u32,
}

impl Default for MorseTimingParams {
    fn default() -> Self {
        Self {
            wpm: 20,
            word_gap_multiplier: 1.0,
            humanization_factor: 0.0,
            random_seed: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MorseRadioParams {
    pub freq_hz: f32,
    pub waveform_type: MorseWaveformType,
    pub background_static_level: f32,
}

impl Default for MorseRadioParams {
    fn default() -> Self {
        Self {
            freq_hz: 440.0,
            waveform_type: MorseWaveformType::Sine,
            background_static_level: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MorseTelegraphParams {
    pub click_sharpness: f32,
    pub resonance_freq: f32,
    pub decay_rate: f32,
    pub mechanical_noise: f32,
    pub solenoid_response: f32,
    pub room_tone_level: f32,
    pub reverb_amount: f32,
}

impl Default for MorseTelegraphParams {
    fn default() -> Self {
        Self {
            click_sharpness: 0.5,
            resonance_freq: 800.0,
            decay_rate: 10.0,
            mechanical_noise: 0.1,
            solenoid_response: 0.7,
            room_tone_level: 0.05,
            reverb_amount: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MorseAudioParams {
    pub sample_rate: i32,
    pub volume: f32,
    pub low_pass_cutoff: f32,
    pub high_pass_cutoff: f32,
    pub audio_mode: MorseAudioMode,
    #[serde(flatten)]
    pub radio_params: MorseRadioParams,
    #[serde(flatten)]
    pub telegraph_params: MorseTelegraphParams,
}

impl Default for MorseAudioParams {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            volume: 0.5,
            low_pass_cutoff: 20000.0,
            high_pass_cutoff: 20.0,
            audio_mode: MorseAudioMode::Radio,
            radio_params: MorseRadioParams::default(),
            telegraph_params: MorseTelegraphParams::default(),
        }
    }
}

// Interpretation types (stubbed for now as requested)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorseSignal {
    pub on: bool,
    pub seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MorseInterpretParams {
    pub max_output_length: i32,
}

impl Default for MorseInterpretParams {
    fn default() -> Self {
        Self {
            max_output_length: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorseInterpretResult {
    pub text: String,
    pub confidence: f32,
    pub signals_processed: i32,
    pub patterns_recognized: i32,
}
