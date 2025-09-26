#include "morse.h"
#include "wav.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

// Generate a test string of specified length
char* generate_test_string(size_t length) {
  char* text = malloc(length + 1);
  if (!text) return NULL;

  const char chars[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 ";
  size_t charset_len = strlen(chars);

  for (size_t i = 0; i < length; i++) {
    text[i] = chars[i % charset_len];
  }
  text[length] = '\0';
  return text;
}

// Get current time in seconds with high precision
double get_time() {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return ts.tv_sec + ts.tv_nsec / 1000000000.0;
}

void stress_test(size_t text_length, const char* size_desc) {
  printf("\n=== Stress test: %s (%zu chars) ===\n", size_desc, text_length);

  // Generate test string
  printf("Generating test string... ");
  fflush(stdout);
  char* text = generate_test_string(text_length);
  if (!text) {
    printf("FAILED - Out of memory\n");
    return;
  }
  printf("OK\n");

  // Allocate timing buffer (10x text length should be safe)
  size_t max_elements = text_length * 10;
  MorseElement* elements = malloc(max_elements * sizeof(MorseElement));
  if (!elements) {
    printf("FAILED - Could not allocate timing buffer\n");
    free(text);
    return;
  }

  // Test timing generation
  printf("Testing morse_timing... ");
  fflush(stdout);
  MorseTimingParams timing_params = MORSE_DEFAULT_TIMING_PARAMS;

  double start = get_time();
  size_t element_count = morse_timing(elements, max_elements, text, &timing_params);
  double timing_duration = get_time() - start;

  if (element_count == 0) {
    printf("FAILED - No elements generated\n");
    free(elements);
    free(text);
    return;
  }

  printf("OK - %.6f seconds (%zu elements, %.2f MB/s)\n",
         timing_duration, element_count,
         (text_length / 1024.0 / 1024.0) / timing_duration);

  // Calculate total audio duration
  double total_duration = 0;
  for (size_t i = 0; i < element_count; i++) {
    total_duration += elements[i].duration_seconds;
  }

  // Test audio generation (limit to reasonable size to avoid memory issues)
  printf("Testing morse_audio... ");
  fflush(stdout);

  MorseAudioParams audio_params = MORSE_DEFAULT_AUDIO_PARAMS;
  audio_params.sample_rate = 22050; // Lower sample rate for stress test

  size_t max_samples = (size_t)(total_duration * audio_params.sample_rate);

  // Limit audio buffer size to prevent memory exhaustion
  size_t max_reasonable_samples = 100 * 1024 * 1024; // 100M samples max
  if (max_samples > max_reasonable_samples) {
    printf("SKIPPED - Would require %.2f GB audio buffer\n",
           (max_samples * sizeof(float)) / 1024.0 / 1024.0 / 1024.0);
  } else {
    float* audio_buffer = malloc(max_samples * sizeof(float));
    if (!audio_buffer) {
      printf("FAILED - Could not allocate audio buffer (%.2f MB)\n",
             (max_samples * sizeof(float)) / 1024.0 / 1024.0);
    } else {
      start = get_time();
      size_t samples = morse_audio(elements, element_count, audio_buffer, max_samples, &audio_params);
      double audio_duration = get_time() - start;

      printf("OK - %.6f seconds (%zu samples, %.2f MB/s)\n",
             audio_duration, samples,
             (text_length / 1024.0 / 1024.0) / audio_duration);

      free(audio_buffer);
    }
  }

  free(elements);
  free(text);
}

int main() {
  printf("Morse Code Stress Test\n");
  printf("======================\n");

  // Test sizes from 10 chars to 100M chars
  struct {
    size_t size;
    const char* desc;
  } test_sizes[] = {
    {10, "10 chars"},
    {100, "100 chars"},
    {1000, "1K chars"},
    {10000, "10K chars"},
    {100000, "100K chars"},
    {1000000, "1M chars"},
    {10000000, "10M chars"},
    {100000000, "100M chars"},
    // 1G is probably too much for most systems
    // {1000000000, "1G chars"}
  };

  size_t num_tests = sizeof(test_sizes) / sizeof(test_sizes[0]);

  for (size_t i = 0; i < num_tests; i++) {
    stress_test(test_sizes[i].size, test_sizes[i].desc);
  }

  printf("\nStress test complete!\n");
  return 0;
}
