#ifndef GENERATE_MORSE_H
#define GENERATE_MORSE_H

typedef struct {
  int wpm;
  float word_gap_multiplier;
  float humanization_factor;
  unsigned int random_seed;
} MorseTimingParams;

typedef struct {
  float freq_hz;
  MorseWaveformType waveform_type;
  float background_static_level;
} MorseRadioParams;

typedef struct {
  float click_sharpness;
  float resonance_freq;
  float decay_rate;
  float mechanical_noise;
  float solenoid_response;
  float room_tone_level;
  float reverb_amount;
} MorseTelegraphParams;

typedef struct {
  int sample_rate;
  float volume;
  float low_pass_cutoff;
  float high_pass_cutoff;
  MorseAudioMode audio_mode;
  union {
    MorseRadioParams radio;
    MorseTelegraphParams telegraph;
  } mode_params;
} MorseAudioParams;

#define MORSE_DEFAULT_TIMING_PARAMS (MorseTimingParams){.wpm = 20, .word_gap_multiplier = 1.0f, .humanization_factor = 0.0f, .random_seed = 0}
#define MORSE_DEFAULT_AUDIO_PARAMS (MorseAudioParams){ \
  .sample_rate = 44100, \
  .volume = 0.5f, \
  .low_pass_cutoff = 20000.0f, \
  .high_pass_cutoff = 20.0f, \
  .audio_mode = MORSE_RADIO, \
  .mode_params.radio = {.freq_hz = 440.0f, .waveform_type = MORSE_WAVEFORM_SINE, .background_static_level = 0.0f} \
}

#define MORSE_DEFAULT_TELEGRAPH_PARAMS (MorseTelegraphParams){ \
  .click_sharpness = 0.5f, \
  .resonance_freq = 800.0f, \
  .decay_rate = 10.0f, \
  .mechanical_noise = 0.1f, \
  .solenoid_response = 0.7f, \
  .room_tone_level = 0.05f, \
  .reverb_amount = 0.3f \
}

size_t morse_timing(MorseElement *out_elements, size_t max_elements, const char *text, const MorseTimingParams *params);
size_t morse_timing_size(const char *text, const MorseTimingParams *params);
size_t morse_audio(const MorseElement *events, size_t element_count, float *out_buffer, size_t max_samples, const MorseAudioParams *params);
size_t morse_audio_size(const MorseElement *events, size_t element_count, const MorseAudioParams *params);
int write_wav_file(const char *filename, const float *samples, size_t sample_count, int sample_rate);

#endif