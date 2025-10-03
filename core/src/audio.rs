use crate::types::{
    MorseAudioMode, MorseAudioParams, MorseElement, MorseElementType, MorseWaveformType,
};
use std::f32::consts::PI;

// Audio constants
const ATTACK_MS: f32 = 5.0; // Envelope attack time to prevent audio clicks
const RELEASE_MS: f32 = 5.0; // Envelope release time to prevent audio clicks
const TELEGRAPH_CLICK_DURATION_SEC: f32 = 0.010; // 10ms click duration
const SQRT2: f32 = std::f32::consts::SQRT_2;

// Simple PRNG for noise generation
struct AudioRng {
    state: u32,
}

impl AudioRng {
    fn new() -> Self {
        Self { state: 12345 }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state >> 16) as f32 / 32768.0 - 1.0 // Range [-1, 1]
    }
}

// Biquad filter structure
#[derive(Clone, Default)]
struct BiquadFilter {
    a0: f32,
    a1: f32,
    a2: f32,
    b1: f32,
    b2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    fn new_lowpass(cutoff_freq: f32, sample_rate: f32) -> Self {
        let mut filter = Self::default();

        if cutoff_freq >= sample_rate * 0.49 {
            // Bypass filter if cutoff is too high
            filter.a0 = 1.0;
            filter.a1 = 0.0;
            filter.a2 = 0.0;
            filter.b1 = 0.0;
            filter.b2 = 0.0;
        } else {
            let w = 2.0 * PI * cutoff_freq / sample_rate;
            let cos_w = w.cos();
            let sin_w = w.sin();
            let alpha = sin_w / SQRT2; // Q = 0.707 for Butterworth

            let norm = 1.0 + alpha;
            filter.a0 = (1.0 - cos_w) / (2.0 * norm);
            filter.a1 = (1.0 - cos_w) / norm;
            filter.a2 = (1.0 - cos_w) / (2.0 * norm);
            filter.b1 = (-2.0 * cos_w) / norm;
            filter.b2 = (1.0 - alpha) / norm;
        }

        filter
    }

    fn new_highpass(cutoff_freq: f32, sample_rate: f32) -> Self {
        let mut filter = Self::default();

        if cutoff_freq <= 1.0 {
            // Bypass filter if cutoff is too low
            filter.a0 = 1.0;
            filter.a1 = 0.0;
            filter.a2 = 0.0;
            filter.b1 = 0.0;
            filter.b2 = 0.0;
        } else {
            let w = 2.0 * PI * cutoff_freq / sample_rate;
            let cos_w = w.cos();
            let sin_w = w.sin();
            let alpha = sin_w / SQRT2; // Q = 0.707 for Butterworth

            let norm = 1.0 + alpha;
            filter.a0 = (1.0 + cos_w) / (2.0 * norm);
            filter.a1 = -(1.0 + cos_w) / norm;
            filter.a2 = (1.0 + cos_w) / (2.0 * norm);
            filter.b1 = (-2.0 * cos_w) / norm;
            filter.b2 = (1.0 - alpha) / norm;
        }

        filter
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.a0 * input + self.a1 * self.x1 + self.a2 * self.x2
            - self.b1 * self.y1
            - self.b2 * self.y2;

        // Update state
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

// Waveform generation
fn generate_waveform(waveform_type: MorseWaveformType, frequency: f32, time: f32) -> f32 {
    let phase = 2.0 * PI * frequency * time;

    match waveform_type {
        MorseWaveformType::Sine => phase.sin(),

        MorseWaveformType::Square => {
            if phase.sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }

        MorseWaveformType::Sawtooth => {
            let normalized_phase = phase % (2.0 * PI);
            (normalized_phase / PI) - 1.0
        }

        MorseWaveformType::Triangle => {
            let normalized_phase = phase % (2.0 * PI);
            if normalized_phase <= PI {
                (2.0 * normalized_phase / PI) - 1.0 // Rising edge: -1 to 1
            } else {
                3.0 - (2.0 * normalized_phase / PI) // Falling edge: 1 to -1
            }
        }
    }
}

// Room tone generation (filtered noise)
struct RoomToneGenerator {
    prev_sample: f32,
    rng: AudioRng,
}

impl RoomToneGenerator {
    fn new() -> Self {
        Self {
            prev_sample: 0.0,
            rng: AudioRng::new(),
        }
    }

    fn generate(&mut self) -> f32 {
        // White noise base
        let white = self.rng.next_f32() * 0.6;

        // Add some low-frequency content (simple 1-pole lowpass)
        let alpha = 0.02; // Very gentle filtering
        self.prev_sample = self.prev_sample * (1.0 - alpha) + white * alpha;

        // Mix white noise with filtered version for warmth
        white * 0.3 + self.prev_sample * 0.7
    }
}

// Radio mode audio generation
fn morse_audio_radio(
    events: &[MorseElement],
    params: &MorseAudioParams,
) -> Result<Vec<f32>, String> {
    let radio = &params.radio_params;
    if radio.freq_hz <= 0.0 || radio.freq_hz > 20000.0 {
        return Err("Invalid frequency".to_string());
    }

    let clamped_volume = params.volume.clamp(0.0, 1.0);

    // Initialize filters
    let mut lowpass = BiquadFilter::new_lowpass(params.low_pass_cutoff, params.sample_rate as f32);
    let mut highpass =
        BiquadFilter::new_highpass(params.high_pass_cutoff, params.sample_rate as f32);
    let mut rng = AudioRng::new();

    let mut samples = Vec::new();

    for elem in events {
        let elem_samples = (elem.duration_seconds * params.sample_rate as f32) as usize;

        if elem.element_type == MorseElementType::Gap {
            // Generate gap samples (silence with optional static)
            for _ in 0..elem_samples {
                let mut signal = 0.0;

                // Add background static if enabled
                if radio.background_static_level > 0.0 {
                    signal = rng.next_f32() * radio.background_static_level * clamped_volume;
                }

                // Apply filters
                let filtered = highpass.process(signal);
                let output = lowpass.process(filtered);
                samples.push(output);
            }
        } else {
            // Generate tone with envelope
            let attack_samples = ((ATTACK_MS / 1000.0) * params.sample_rate as f32) as usize;
            let release_samples = ((RELEASE_MS / 1000.0) * params.sample_rate as f32) as usize;

            // Clamp envelope lengths to element duration
            let attack_samples = attack_samples.min(elem_samples / 2);
            let release_samples = release_samples.min(elem_samples / 2);
            let _sustain_start = attack_samples;
            let release_start = elem_samples.saturating_sub(release_samples);

            for j in 0..elem_samples {
                let t = j as f32 / params.sample_rate as f32;
                let mut envelope = 1.0;

                // Calculate envelope
                if j < attack_samples {
                    envelope = j as f32 / attack_samples as f32;
                } else if j >= release_start {
                    envelope = (elem_samples - j) as f32 / release_samples as f32;
                }

                let waveform = generate_waveform(radio.waveform_type, radio.freq_hz, t);
                let mut signal = waveform * clamped_volume * envelope;

                // Add background static if enabled
                if radio.background_static_level > 0.0 {
                    signal += rng.next_f32() * radio.background_static_level * clamped_volume;
                }

                // Apply filters
                let filtered = highpass.process(signal);
                let output = lowpass.process(filtered);
                samples.push(output);
            }
        }
    }

    Ok(samples)
}

// Telegraph click generation with mechanical resonance
fn generate_telegraph_click(
    t: f32,
    telegraph: &crate::types::MorseTelegraphParams,
    freq_multiplier: f32,
    sharpness_multiplier: f32,
    volume_multiplier: f32,
) -> f32 {
    let actual_freq = telegraph.resonance_freq * freq_multiplier;

    // Calculate pitch variation with mechanical noise
    let pitch_variation = if telegraph.mechanical_noise > 0.0 {
        // Simple noise approximation - in production you'd use proper PRNG
        let noise = (t * 1234.5).sin() * 2.0 - 1.0; // -1 to 1
        1.0 + noise * telegraph.mechanical_noise * 0.05 // ±5% max variation
    } else {
        1.0
    };
    let actual_freq = actual_freq * pitch_variation;

    // Generate composite resonance signal
    let primary_resonance = (2.0 * PI * actual_freq * t).sin();
    let secondary_freq = actual_freq * 2.3; // Not exactly harmonic for realism
    let secondary_amplitude = if freq_multiplier == 1.0 { 0.4 } else { 0.3 };
    let secondary_resonance = (2.0 * PI * secondary_freq * t).sin() * secondary_amplitude;

    let signal = primary_resonance + secondary_resonance;

    // Apply exponential decay
    let decay = (-t * telegraph.decay_rate).exp();

    // Apply attack sharpness
    let sharpness_factor = telegraph.click_sharpness.clamp(0.0, 1.0) * 999.0 + 1.0;
    let attack = (-t * sharpness_factor * sharpness_multiplier).exp();

    signal * decay * attack * volume_multiplier
}

// Telegraph mode audio generation (simplified version)
fn morse_audio_telegraph(
    events: &[MorseElement],
    params: &MorseAudioParams,
) -> Result<Vec<f32>, String> {
    let telegraph = &params.telegraph_params;
    let clamped_volume = params.volume.clamp(0.0, 1.0);

    // Initialize filters
    let mut lowpass = BiquadFilter::new_lowpass(params.low_pass_cutoff, params.sample_rate as f32);
    let mut highpass =
        BiquadFilter::new_highpass(params.high_pass_cutoff, params.sample_rate as f32);
    let mut room_tone = RoomToneGenerator::new();

    let mut samples = Vec::new();

    for elem in events {
        let elem_samples = (elem.duration_seconds * params.sample_rate as f32) as usize;

        if elem.element_type == MorseElementType::Gap {
            // Generate gap samples with optional room tone
            for _ in 0..elem_samples {
                let mut signal = 0.0;

                // Add room tone if enabled
                if telegraph.room_tone_level > 0.0 {
                    signal = room_tone.generate() * telegraph.room_tone_level * clamped_volume;
                }

                // Apply filters
                let filtered = highpass.process(signal);
                let output = lowpass.process(filtered);
                samples.push(output);
            }
        } else {
            // Generate telegraph click
            let click_samples = (TELEGRAPH_CLICK_DURATION_SEC * params.sample_rate as f32) as usize;
            let click_samples = click_samples.min(elem_samples);

            for j in 0..elem_samples {
                let t = j as f32 / params.sample_rate as f32;
                let mut signal = 0.0;

                // Generate click at the beginning
                if j < click_samples {
                    signal = generate_telegraph_click(t, telegraph, 1.0, 1.0, clamped_volume);
                }

                // Add room tone if enabled
                if telegraph.room_tone_level > 0.0 {
                    signal += room_tone.generate() * telegraph.room_tone_level * clamped_volume;
                }

                // Apply filters
                let filtered = highpass.process(signal);
                let output = lowpass.process(filtered);
                samples.push(output);
            }
        }
    }

    Ok(samples)
}

/// Generate morse code audio from timing elements
pub fn morse_audio(events: &[MorseElement], params: &MorseAudioParams) -> Result<Vec<f32>, String> {
    if events.is_empty() {
        return Ok(Vec::new());
    }

    if params.sample_rate <= 0 || params.sample_rate > 192000 {
        return Err("Invalid sample rate".to_string());
    }

    match params.audio_mode {
        MorseAudioMode::Radio => morse_audio_radio(events, params),
        MorseAudioMode::Telegraph => morse_audio_telegraph(events, params),
    }
}

/// Calculate the total number of samples needed for the given timing elements
pub fn morse_audio_size(
    events: &[MorseElement],
    params: &MorseAudioParams,
) -> Result<usize, String> {
    if params.sample_rate <= 0 {
        return Err("Invalid sample rate".to_string());
    }

    let total_duration: f32 = events.iter().map(|e| e.duration_seconds).sum();

    Ok((total_duration * params.sample_rate as f32) as usize)
}
