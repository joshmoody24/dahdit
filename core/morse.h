#ifndef MORSE_H
#define MORSE_H

#include <stddef.h>

typedef enum {
  MORSE_DOT,
  MORSE_DASH,
  MORSE_GAP
} MorseElementType;

typedef struct {
  MorseElementType type;
  float duration_seconds;
} MorseElement;

typedef struct {
  int wpm;
  float word_gap_multiplier;    // 1.0 = standard, 2.0 = double word gaps
  float humanization_factor;   // 0.0 = perfect, 1.0 = very human
  unsigned int random_seed;     // Random seed for humanization (0 = use time)
} MorseTimingParams;

// Audio generation modes
typedef enum {
  MORSE_RADIO = 0,     // Radio transmission (continuous wave)
  MORSE_TELEGRAPH = 1  // Telegraph clicks
} MorseAudioMode;

// Radio waveform types
typedef enum {
  MORSE_WAVEFORM_SINE = 0,     // Pure sine wave
  MORSE_WAVEFORM_SQUARE = 1,   // Square wave
  MORSE_WAVEFORM_SAWTOOTH = 2, // Sawtooth wave
  MORSE_WAVEFORM_TRIANGLE = 3  // Triangle wave
} MorseWaveformType;

// Radio mode parameters
typedef struct {
  float freq_hz;                // Tone frequency
  MorseWaveformType waveform_type; // Waveform shape
  float background_static_level; // Static noise level (0.0-1.0)
} MorseRadioParams;

// Telegraph mode parameters
typedef struct {
  float click_sharpness;   // Attack steepness (0.0-1.0, 1.0 = sharpest)
  float resonance_freq;    // Mechanical resonance frequency
  float decay_rate;        // Exponential decay rate
  float mechanical_noise;  // Random variations (0.0-1.0)
} MorseTelegraphParams;

typedef struct {
  int sample_rate;
  float volume;
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
  .audio_mode = MORSE_RADIO, \
  .mode_params.radio = {.freq_hz = 440.0f, .waveform_type = MORSE_WAVEFORM_SINE, .background_static_level = 0.0f} \
}

#define MORSE_DEFAULT_TELEGRAPH_PARAMS (MorseTelegraphParams){ \
  .click_sharpness = 0.5f, \
  .resonance_freq = 800.0f, \
  .decay_rate = 10.0f, \
  .mechanical_noise = 0.1f \
}

size_t morse_timing(MorseElement *out_elements, size_t max_elements, const char *text, const MorseTimingParams *params);

size_t morse_timing_size(const char *text, const MorseTimingParams *params);

size_t morse_audio(const MorseElement *events, size_t element_count, float *out_buffer, size_t max_samples, const MorseAudioParams *params);

size_t morse_audio_size(const MorseElement *events, size_t element_count, const MorseAudioParams *params);

int write_wav_file(const char *filename, const float *samples, size_t sample_count, int sample_rate);

#endif
