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

typedef struct {
  int sample_rate;
  float freq_hz;
  float volume;
} MorseAudioParams;

#define MORSE_DEFAULT_TIMING_PARAMS (MorseTimingParams){.wpm = 20, .word_gap_multiplier = 1.0f, .humanization_factor = 0.0f, .random_seed = 0}
#define MORSE_DEFAULT_AUDIO_PARAMS (MorseAudioParams){.sample_rate = 44100, .freq_hz = 440.0f, .volume = 0.5f}

size_t morse_timing(MorseElement *out_elements, size_t max_elements, const char *text, const MorseTimingParams *params);

size_t morse_timing_size(const char *text, const MorseTimingParams *params);

size_t morse_audio(const MorseElement *events, size_t element_count, float *out_buffer, size_t max_samples, const MorseAudioParams *params);

size_t morse_audio_size(const MorseElement *events, size_t element_count, const MorseAudioParams *params);

int write_wav_file(const char *filename, const float *samples, size_t sample_count, int sample_rate);

#endif
