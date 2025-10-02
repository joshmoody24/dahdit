#include "morse_abi.h"
#include "../../core/morse.h"
#include <stdlib.h>
#include <string.h>

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#define EXPORT EMSCRIPTEN_KEEPALIVE
#else
#define EXPORT
#endif

// Context structure holds all parameter structs
struct MorseCtx {
  MorseTimingParams timing_params;
  MorseAudioParams audio_params;
  MorseInterpretParams interpret_params;
};

EXPORT MorseCtx* morse_new(void) {
  MorseCtx* ctx = malloc(sizeof(MorseCtx));
  if (!ctx) return NULL;

  // Initialize with default values
  ctx->timing_params = MORSE_DEFAULT_TIMING_PARAMS;
  ctx->audio_params = MORSE_DEFAULT_AUDIO_PARAMS;
  ctx->interpret_params = MORSE_DEFAULT_INTERPRET_PARAMS;
  
  return ctx;
}

EXPORT void morse_free(MorseCtx* ctx) {
  if (ctx) {
    free(ctx);
  }
}

EXPORT int morse_set_i32(MorseCtx* ctx, int key, int value) {
  if (!ctx) return 0;

  switch (key) {
    case MORSE_OPT_WPM:
      ctx->timing_params.wpm = value;
      return 1;
    case MORSE_OPT_SAMPLE_RATE:
      ctx->audio_params.sample_rate = value;
      return 1;
    case MORSE_OPT_RANDOM_SEED:
      ctx->timing_params.random_seed = (unsigned int)value;
      return 1;
    case MORSE_OPT_AUDIO_MODE:
      ctx->audio_params.audio_mode = (MorseAudioMode)value;
      return 1;
    case MORSE_OPT_WAVEFORM_TYPE:
      ctx->audio_params.mode_params.radio.waveform_type = (MorseWaveformType)value;
      return 1;
    case MORSE_OPT_MAX_K_MEANS_ITERATIONS:
      ctx->interpret_params.max_k_means_iterations = value;
      return 1;
    case MORSE_OPT_MAX_OUTPUT_LENGTH:
      ctx->interpret_params.max_output_length = value;
      return 1;
    default:
      return 0; // Unknown key - no-op
  }
}

EXPORT int morse_set_f32(MorseCtx* ctx, int key, float value) {
  if (!ctx) return 0;

  switch (key) {
    case MORSE_OPT_FREQ_HZ:
      ctx->audio_params.mode_params.radio.freq_hz = value;
      return 1;
    case MORSE_OPT_VOLUME:
      ctx->audio_params.volume = value;
      return 1;
    case MORSE_OPT_WORD_GAP_MULTIPLIER:
      ctx->timing_params.word_gap_multiplier = value;
      return 1;
    case MORSE_OPT_HUMANIZATION_FACTOR:
      ctx->timing_params.humanization_factor = value;
      return 1;
    case MORSE_OPT_BACKGROUND_STATIC_LEVEL:
      ctx->audio_params.mode_params.radio.background_static_level = value;
      return 1;
    case MORSE_OPT_CLICK_SHARPNESS:
      ctx->audio_params.mode_params.telegraph.click_sharpness = value;
      return 1;
    case MORSE_OPT_RESONANCE_FREQ:
      ctx->audio_params.mode_params.telegraph.resonance_freq = value;
      return 1;
    case MORSE_OPT_DECAY_RATE:
      ctx->audio_params.mode_params.telegraph.decay_rate = value;
      return 1;
    case MORSE_OPT_MECHANICAL_NOISE:
      ctx->audio_params.mode_params.telegraph.mechanical_noise = value;
      return 1;
    case MORSE_OPT_SOLENOID_RESPONSE:
      ctx->audio_params.mode_params.telegraph.solenoid_response = value;
      return 1;
    case MORSE_OPT_ROOM_TONE_LEVEL:
      ctx->audio_params.mode_params.telegraph.room_tone_level = value;
      return 1;
    case MORSE_OPT_REVERB_AMOUNT:
      ctx->audio_params.mode_params.telegraph.reverb_amount = value;
      return 1;
    case MORSE_OPT_LOW_PASS_CUTOFF:
      ctx->audio_params.low_pass_cutoff = value;
      return 1;
    case MORSE_OPT_HIGH_PASS_CUTOFF:
      ctx->audio_params.high_pass_cutoff = value;
      return 1;
    case MORSE_OPT_CONVERGENCE_THRESHOLD:
      ctx->interpret_params.convergence_threshold = value;
      return 1;
    case MORSE_OPT_NOISE_THRESHOLD:
      ctx->interpret_params.noise_threshold = value;
      return 1;
    default:
      return 0; // Unknown key - no-op
  }
}

EXPORT int morse_set_str(MorseCtx* ctx, int key, const char* value) {
  if (!ctx || !value) return 0;

  // Reserved for future string options
  (void)key; // Suppress unused parameter warning
  return 0; // No string options implemented yet
}

EXPORT size_t morse_timing_size_ctx(MorseCtx* ctx, const char* text) {
  if (!ctx) return 0;
  return morse_timing_size(text, &ctx->timing_params);
}

EXPORT size_t morse_timing_fill_ctx(MorseCtx* ctx, const char* text, int* types, float* durs, size_t max) {
  if (!ctx || !types || !durs) return 0;

  // Allocate temporary MorseElement array
  MorseElement* elements = malloc(max * sizeof(MorseElement));
  if (!elements) return 0;

  // Generate timing using core function
  size_t count = morse_timing(elements, max, text, &ctx->timing_params);

  // Unpack MorseElement structs into separate arrays
  for (size_t i = 0; i < count; i++) {
    types[i] = (int)elements[i].type;
    durs[i] = elements[i].duration_seconds;
  }

  free(elements);
  return count;
}

EXPORT size_t morse_audio_size_ctx(MorseCtx* ctx, const int* types, const float* durs, size_t n) {
  if (!ctx || !types || !durs) return 0;

  // Pack arrays into temporary MorseElement array
  MorseElement* elements = malloc(n * sizeof(MorseElement));
  if (!elements) return 0;

  for (size_t i = 0; i < n; i++) {
    elements[i].type = (MorseElementType)types[i];
    elements[i].duration_seconds = durs[i];
  }

  size_t size = morse_audio_size(elements, n, &ctx->audio_params);
  free(elements);
  return size;
}

EXPORT size_t morse_audio_fill_ctx(MorseCtx* ctx, const int* types, const float* durs, size_t n, float* samples, size_t max) {
  if (!ctx || !types || !durs || !samples) return 0;

  // Pack arrays into temporary MorseElement array
  MorseElement* elements = malloc(n * sizeof(MorseElement));
  if (!elements) return 0;

  for (size_t i = 0; i < n; i++) {
    elements[i].type = (MorseElementType)types[i];
    elements[i].duration_seconds = durs[i];
  }

  size_t count = morse_audio(elements, n, samples, max, &ctx->audio_params);
  free(elements);
  return count;
}

EXPORT size_t morse_interpret_size_ctx(MorseCtx* ctx, const int* on_states, const float* durations, size_t signal_count) {
  if (!ctx || !on_states || !durations) return 0;

  // Pack arrays into temporary MorseSignal array
  MorseSignal* signals = malloc(signal_count * sizeof(MorseSignal));
  if (!signals) return 0;

  for (size_t i = 0; i < signal_count; i++) {
    signals[i].on = on_states[i] != 0;
    signals[i].seconds = durations[i];
  }

  size_t size = morse_interpret_text_size(signals, signal_count, &ctx->interpret_params);
  free(signals);
  return size;
}

EXPORT size_t morse_interpret_fill_ctx(MorseCtx* ctx, const int* on_states, const float* durations, size_t signal_count, char* text, size_t max_text_size, float* confidence, int* signals_processed, int* patterns_recognized) {
  if (!ctx || !on_states || !durations || !text) return 0;

  // Pack arrays into temporary MorseSignal array
  MorseSignal* signals = malloc(signal_count * sizeof(MorseSignal));
  if (!signals) return 0;

  for (size_t i = 0; i < signal_count; i++) {
    signals[i].on = on_states[i] != 0;
    signals[i].seconds = durations[i];
  }

  MorseInterpretResult result = morse_interpret(signals, signal_count, &ctx->interpret_params);
  free(signals);

  if (!result.text) return 0;

  // Copy text to output buffer
  size_t text_len = result.text_length;
  if (text_len >= max_text_size) {
    text_len = max_text_size - 1; // Leave room for null terminator
  }

  memcpy(text, result.text, text_len);
  text[text_len] = '\0';

  // Set output parameters if provided
  if (confidence) *confidence = result.confidence;
  if (signals_processed) *signals_processed = result.signals_processed;
  if (patterns_recognized) *patterns_recognized = result.patterns_recognized;

  morse_interpret_result_free(&result);
  return text_len;
}