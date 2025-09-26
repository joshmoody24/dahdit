#ifndef WAV_H
#define WAV_H

#include <stddef.h>

#define WAV_SUCCESS 0
#define WAV_FILE_ERROR -1
#define WAV_WRITE_ERROR -2

int write_wav_file(const char *filename, const float *samples, size_t sample_count, int sample_rate);

#endif