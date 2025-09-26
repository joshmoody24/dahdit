#include "morse.h"
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <assert.h>

// Test result tracking
static int tests_run = 0;
static int tests_passed = 0;

#define TEST(name) \
  do { \
    tests_run++; \
    printf("Running %s... ", #name); \
    if (test_##name()) { \
      tests_passed++; \
      printf("PASS\n"); \
    } else { \
      printf("FAIL\n"); \
    } \
  } while(0)

// Test morse_timing with simple known cases
static int test_basic_timing() {
  MorseTimingParams params = MORSE_DEFAULT_TIMING_PARAMS;
  MorseElement elements[100];

  // Test "E" (single dot)
  size_t count = morse_timing(elements, 100, "E", &params);
  if (count != 1) return 0;
  if (elements[0].type != MORSE_DOT) return 0;

  // Test "T" (single dash)
  count = morse_timing(elements, 100, "T", &params);
  if (count != 1) return 0;
  if (elements[0].type != MORSE_DASH) return 0;

  // Test "A" (dot-dash with gap between)
  count = morse_timing(elements, 100, "A", &params);
  if (count != 3) return 0; // dot + gap + dash
  if (elements[0].type != MORSE_DOT) return 0;
  if (elements[1].type != MORSE_GAP) return 0;
  if (elements[2].type != MORSE_DASH) return 0;

  return 1;
}

// Test morse_timing with spaces (inter-word gaps)
static int test_spacing() {
  MorseTimingParams params = MORSE_DEFAULT_TIMING_PARAMS;
  MorseElement elements[100];

  // Test "E E" (two E's with word gap)
  size_t count = morse_timing(elements, 100, "E E", &params);
  if (count != 3) return 0; // dot + word_gap + dot
  if (elements[0].type != MORSE_DOT) return 0;
  if (elements[1].type != MORSE_GAP) return 0;
  if (elements[2].type != MORSE_DOT) return 0;

  // Word gap should be 7x dot duration
  float dot_duration = elements[0].duration_seconds;
  float word_gap_duration = elements[1].duration_seconds;
  if (fabs(word_gap_duration - dot_duration * 7.0f) > 0.001f) return 0;

  return 1;
}

// Test prosign syntax [SOS]
static int test_prosigns() {
  MorseTimingParams params = MORSE_DEFAULT_TIMING_PARAMS;
  MorseElement elements[100];

  // Test "[SOS]" - should have 1-dot gaps between chars instead of 3-dot
  size_t count = morse_timing(elements, 100, "[SOS]", &params);
  if (count == 0) return 0;

  // Should contain dots, dashes, and gaps
  int has_dots = 0, has_dashes = 0, has_gaps = 0;
  for (size_t i = 0; i < count; i++) {
    if (elements[i].type == MORSE_DOT) has_dots = 1;
    if (elements[i].type == MORSE_DASH) has_dashes = 1;
    if (elements[i].type == MORSE_GAP) has_gaps = 1;
  }

  return has_dots && has_dashes && has_gaps;
}

// Test morse_audio basic functionality
static int test_audio_generation() {
  MorseTimingParams timing_params = MORSE_DEFAULT_TIMING_PARAMS;
  MorseAudioParams audio_params = MORSE_DEFAULT_AUDIO_PARAMS;
  audio_params.sample_rate = 8000; // Low sample rate for test

  MorseElement elements[10];
  float audio_buffer[1000];

  // Generate timing for "E"
  size_t element_count = morse_timing(elements, 10, "E", &timing_params);
  if (element_count == 0) return 0;

  // Generate audio
  size_t samples = morse_audio(elements, element_count, audio_buffer, 1000, &audio_params);
  if (samples == 0) return 0;

  // Audio should contain non-zero values
  int has_nonzero = 0;
  for (size_t i = 0; i < samples; i++) {
    if (audio_buffer[i] != 0.0f) {
      has_nonzero = 1;
      break;
    }
  }

  return has_nonzero;
}

// Test input validation
static int test_input_validation() {
  MorseTimingParams params = MORSE_DEFAULT_TIMING_PARAMS;
  MorseElement elements[10];

  // Test NULL inputs
  if (morse_timing(NULL, 10, "E", &params) != 0) return 0;
  if (morse_timing(elements, 10, NULL, &params) != 0) return 0;
  if (morse_timing(elements, 10, "E", NULL) != 0) return 0;

  // Test empty string
  if (morse_timing(elements, 10, "", &params) != 0) return 0;

  return 1;
}

// Test buffer overflow protection
static int test_buffer_limits() {
  MorseTimingParams params = MORSE_DEFAULT_TIMING_PARAMS;
  MorseElement elements[5]; // Small buffer

  // Test with text that would overflow
  size_t count = morse_timing(elements, 5, "ABCDEFG", &params);
  // Should not exceed buffer size
  if (count > 5) return 0;

  return 1;
}

// Test WPM timing accuracy
static int test_wpm_timing() {
  MorseTimingParams fast_params = {.wpm = 40};
  MorseTimingParams slow_params = {.wpm = 10};
  MorseElement fast_elements[10], slow_elements[10];

  // Generate "E" at different speeds
  morse_timing(fast_elements, 10, "E", &fast_params);
  morse_timing(slow_elements, 10, "E", &slow_params);

  // Fast WPM should produce shorter durations
  if (fast_elements[0].duration_seconds >= slow_elements[0].duration_seconds) return 0;

  // Duration ratio should be roughly inverse of WPM ratio
  float duration_ratio = slow_elements[0].duration_seconds / fast_elements[0].duration_seconds;
  float wpm_ratio = 40.0f / 10.0f; // fast/slow

  // Allow 10% tolerance
  if (fabs(duration_ratio - wpm_ratio) > wpm_ratio * 0.1f) return 0;

  return 1;
}

int main() {
  printf("Morse Code Unit Tests\n");
  printf("====================\n\n");

  TEST(basic_timing);
  TEST(spacing);
  TEST(prosigns);
  TEST(audio_generation);
  TEST(input_validation);
  TEST(buffer_limits);
  TEST(wpm_timing);

  printf("\nResults: %d/%d tests passed\n", tests_passed, tests_run);

  if (tests_passed == tests_run) {
    printf("All tests PASSED!\n");
    return 0;
  } else {
    printf("Some tests FAILED!\n");
    return 1;
  }
}