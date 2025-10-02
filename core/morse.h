#ifndef MORSE_H
#define MORSE_H

#include <stddef.h>
#include <stdbool.h>

typedef enum {
  MORSE_DOT,
  MORSE_DASH,
  MORSE_GAP
} MorseElementType;

typedef struct {
  MorseElementType type;
  float duration_seconds;
} MorseElement;

typedef enum {
  MORSE_RADIO = 0,
  MORSE_TELEGRAPH = 1
} MorseAudioMode;

typedef enum {
  MORSE_WAVEFORM_SINE = 0,
  MORSE_WAVEFORM_SQUARE = 1,
  MORSE_WAVEFORM_SAWTOOTH = 2,
  MORSE_WAVEFORM_TRIANGLE = 3
} MorseWaveformType;

#include "generate_morse.h"
#include "interpret_morse.h"

#endif
