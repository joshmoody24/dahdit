#include "morse.h"
#include "wav.h"
#include <stdio.h>

int main() {
  MorseTimingParams timing_params = MORSE_DEFAULT_TIMING_PARAMS;
  MorseElement events[100];
  size_t event_count = morse_timing(events, 100, "HELLO", &timing_params);

  MorseAudioParams audio_params = MORSE_DEFAULT_AUDIO_PARAMS;
  audio_params.sample_rate /= 2;
  float audio_buffer[44100 * 5];

  size_t samples = morse_audio(events, event_count, audio_buffer, sizeof(audio_buffer)/sizeof(audio_buffer[0]), &audio_params);

  printf("Generated %zu audio samples for %zu Morse events.\n", samples, event_count);

  int wav_result = write_wav_file("output.wav", audio_buffer, samples, audio_params.sample_rate);
  if (wav_result == WAV_SUCCESS) {
    printf("Saved audio to output.wav\n");
  } else {
    perror("Failed to save WAV file");
  }

  return 0;
}
