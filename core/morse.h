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
} MorseTimingParams;

typedef struct {
  int sample_rate;
  float freq_hz;
  float volume;
} MorseAudioParams;

#define MORSE_DEFAULT_TIMING_PARAMS (MorseTimingParams){.wpm = 20}
#define MORSE_DEFAULT_AUDIO_PARAMS (MorseAudioParams){.sample_rate = 44100, .freq_hz = 440.0f, .volume = 0.5f}

size_t morse_timing(MorseElement *out_elements, size_t max_elements, const char *text, const MorseTimingParams *params);

size_t morse_audio(const MorseElement *events, size_t element_count, float *out_buffer, size_t max_samples, const MorseAudioParams *params);

int write_wav_file(const char *filename, const float *samples, size_t sample_count, int sample_rate);

#endif
