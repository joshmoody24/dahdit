#ifndef MORSE_ABI_H
#define MORSE_ABI_H

#include <stddef.h>

// Opaque context type
typedef struct MorseCtx MorseCtx;

// Option keys for context configuration
enum MorseOptionKey {
  MORSE_OPT_WPM = 0,
  MORSE_OPT_SAMPLE_RATE = 1,
  MORSE_OPT_FREQ_HZ = 2,
  MORSE_OPT_VOLUME = 3,
  MORSE_OPT_WORD_GAP_MULTIPLIER = 4,
  MORSE_OPT_HUMANIZATION_FACTOR = 5,
  MORSE_OPT_RANDOM_SEED = 6
};

// Context management
MorseCtx* morse_new(void);
void morse_free(MorseCtx* ctx);

// Option setters
int morse_set_i32(MorseCtx* ctx, int key, int value);
int morse_set_f32(MorseCtx* ctx, int key, float value);
int morse_set_str(MorseCtx* ctx, int key, const char* value);  // Reserved for future options

// Timing functions
size_t morse_timing_size_ctx(MorseCtx* ctx, const char* text);
size_t morse_timing_fill_ctx(MorseCtx* ctx, const char* text, int* types, float* durs, size_t max);

// Audio functions  
size_t morse_audio_size_ctx(MorseCtx* ctx, const int* types, const float* durs, size_t n);
size_t morse_audio_fill_ctx(MorseCtx* ctx, const int* types, const float* durs, size_t n, float* samples, size_t max);

#endif