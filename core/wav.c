#include "wav.h"
#include <stdio.h>
#include <stdint.h>
#include <string.h>

int write_wav_file(const char *filename, const float *samples, size_t sample_count, int sample_rate) {
  FILE *file = fopen(filename, "wb");
  if (!file) return WAV_FILE_ERROR;

  // WAV header structure
  uint32_t data_size = sample_count * 2; // 16-bit samples
  uint32_t file_size = data_size + 36;

  // RIFF header
  fwrite("RIFF", 1, 4, file);
  fwrite(&file_size, 4, 1, file);
  fwrite("WAVE", 1, 4, file);

  // Format chunk
  fwrite("fmt ", 1, 4, file);
  uint32_t fmt_size = 16;
  fwrite(&fmt_size, 4, 1, file);
  uint16_t audio_format = 1; // PCM
  fwrite(&audio_format, 2, 1, file);
  uint16_t channels = 1;
  fwrite(&channels, 2, 1, file);
  uint32_t sample_rate_32 = sample_rate;
  fwrite(&sample_rate_32, 4, 1, file);
  uint32_t byte_rate = sample_rate * channels * 2;
  fwrite(&byte_rate, 4, 1, file);
  uint16_t block_align = channels * 2;
  fwrite(&block_align, 2, 1, file);
  uint16_t bits_per_sample = 16;
  fwrite(&bits_per_sample, 2, 1, file);

  // Data chunk
  fwrite("data", 1, 4, file);
  fwrite(&data_size, 4, 1, file);

  // Convert float samples to 16-bit PCM and write
  for (size_t i = 0; i < sample_count; i++) {
    int16_t sample = (int16_t)(samples[i] * 32767.0f);
    fwrite(&sample, 2, 1, file);
  }

  fclose(file);
  return WAV_SUCCESS;
}