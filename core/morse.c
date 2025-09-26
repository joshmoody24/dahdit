#include "morse.h"
#include <math.h>
#include <stdlib.h>
#include <time.h>

const float DOT_LENGTH_WPM = 1.2f;      // Standard ITU timing formula: dot duration = 1.2 / WPM seconds
const int DOTS_PER_DASH = 3;           // ITU specification: dash = 3 dot durations
const int DOTS_PER_CHAR_GAP = 3;       // ITU specification: inter-character gap = 3 dot durations
const int DOTS_PER_WORD_GAP = 7;       // ITU specification: inter-word gap = 7 dot durations
const float ATTACK_MS = 5.0f;          // Envelope attack time to prevent audio clicks
const float RELEASE_MS = 5.0f;         // Envelope release time to prevent audio clicks
const float HUMANIZATION_MAX_VARIANCE = 0.3f;  // Maximum timing variation as fraction of base duration

// Telegraph mode constants
const float TELEGRAPH_CLICK_DURATION_SEC = 0.010f;  // 10ms click duration
const float TELEGRAPH_MIN_SHARPNESS = 1.0f;         // Minimum attack sharpness factor
const float TELEGRAPH_MAX_SHARPNESS = 1000.0f;      // Maximum attack sharpness factor

// White noise generator
static float generate_white_noise(void) {
  // Simple white noise using rand() - normalized to [-1, 1]
  return (2.0f * (float)rand() / (float)RAND_MAX) - 1.0f;
}

// Waveform generation functions
static float generate_waveform(MorseWaveformType waveform_type, float frequency, float time) {
  float phase = 2.0f * M_PI * frequency * time;

  switch (waveform_type) {
    case MORSE_WAVEFORM_SINE:
      return sinf(phase);

    case MORSE_WAVEFORM_SQUARE:
      return sinf(phase) >= 0.0f ? 1.0f : -1.0f;

    case MORSE_WAVEFORM_SAWTOOTH:
      // Normalize phase to [0, 2π] then map to [-1, 1]
      phase = fmodf(phase, 2.0f * M_PI);
      return (phase / M_PI) - 1.0f;

    case MORSE_WAVEFORM_TRIANGLE:
      phase = fmodf(phase, 2.0f * M_PI);
      if (phase <= M_PI) {
        return (2.0f * phase / M_PI) - 1.0f;  // Rising edge: -1 to 1
      } else {
        return 3.0f - (2.0f * phase / M_PI);  // Falling edge: 1 to -1
      }

    default:
      return sinf(phase);  // Fallback to sine
  }
}

// Simple humanization - adds random variation to timing with bounded output
static float apply_humanization(float base_duration, float humanization_factor) {
  if (humanization_factor <= 0.0f) return base_duration;

  // Generate random variation: ±(humanization_factor * HUMANIZATION_MAX_VARIANCE) of base duration
  float max_variation = base_duration * humanization_factor * HUMANIZATION_MAX_VARIANCE;
  float variation = ((float)rand() / (float)RAND_MAX - 0.5f) * 2.0f * max_variation;

  float result = base_duration + variation;

  // Clamp result to safe bounds: [10% of base, base * (1 + max_variance)]
  float min_duration = base_duration * 0.1f;
  float max_duration = base_duration * (1.0f + humanization_factor * HUMANIZATION_MAX_VARIANCE);

  if (result < min_duration) return min_duration;
  if (result > max_duration) return max_duration;
  return result;
}


// Morse code patterns using direct array indexing (O(1) lookup)
// Pattern format: dots=0, dashes=1, terminated by -1
static const int pattern_A[] = {0, 1, -1};                    // .-
static const int pattern_B[] = {1, 0, 0, 0, -1};              // -...
static const int pattern_C[] = {1, 0, 1, 0, -1};              // -.-.
static const int pattern_D[] = {1, 0, 0, -1};                 // -..
static const int pattern_E[] = {0, -1};                       // .
static const int pattern_F[] = {0, 0, 1, 0, -1};              // ..-.
static const int pattern_G[] = {1, 1, 0, -1};                 // --.
static const int pattern_H[] = {0, 0, 0, 0, -1};              // ....
static const int pattern_I[] = {0, 0, -1};                    // ..
static const int pattern_J[] = {0, 1, 1, 1, -1};              // .---
static const int pattern_K[] = {1, 0, 1, -1};                 // -.-
static const int pattern_L[] = {0, 1, 0, 0, -1};              // .-..
static const int pattern_M[] = {1, 1, -1};                    // --
static const int pattern_N[] = {1, 0, -1};                    // -.
static const int pattern_O[] = {1, 1, 1, -1};                 // ---
static const int pattern_P[] = {0, 1, 1, 0, -1};              // .--.
static const int pattern_Q[] = {1, 1, 0, 1, -1};              // --.-
static const int pattern_R[] = {0, 1, 0, -1};                 // .-.
static const int pattern_S[] = {0, 0, 0, -1};                 // ...
static const int pattern_T[] = {1, -1};                       // -
static const int pattern_U[] = {0, 0, 1, -1};                 // ..-
static const int pattern_V[] = {0, 0, 0, 1, -1};              // ...-
static const int pattern_W[] = {0, 1, 1, -1};                 // .--
static const int pattern_X[] = {1, 0, 0, 1, -1};              // -..-
static const int pattern_Y[] = {1, 0, 1, 1, -1};              // -.--
static const int pattern_Z[] = {1, 1, 0, 0, -1};              // --..

static const int pattern_0[] = {1, 1, 1, 1, 1, -1};           // -----
static const int pattern_1[] = {0, 1, 1, 1, 1, -1};           // .----
static const int pattern_2[] = {0, 0, 1, 1, 1, -1};           // ..---
static const int pattern_3[] = {0, 0, 0, 1, 1, -1};           // ...--
static const int pattern_4[] = {0, 0, 0, 0, 1, -1};           // ....-
static const int pattern_5[] = {0, 0, 0, 0, 0, -1};           // .....
static const int pattern_6[] = {1, 0, 0, 0, 0, -1};           // -....
static const int pattern_7[] = {1, 1, 0, 0, 0, -1};           // --...
static const int pattern_8[] = {1, 1, 1, 0, 0, -1};           // ---..
static const int pattern_9[] = {1, 1, 1, 1, 0, -1};           // ----.

static const int pattern_period[] = {0, 1, 0, 1, 0, 1, -1};   // .-.-.-
static const int pattern_comma[] = {1, 1, 0, 0, 1, 1, -1};    // --..--
static const int pattern_question[] = {0, 0, 1, 1, 0, 0, -1}; // ..--..
static const int pattern_quote[] = {0, 1, 1, 1, 1, 0, -1};    // .----.
static const int pattern_exclaim[] = {1, 0, 1, 0, 1, 1, -1};  // -.-.--
static const int pattern_slash[] = {1, 0, 0, 1, 0, -1};       // -..-.
static const int pattern_lparen[] = {1, 0, 1, 1, 0, -1};      // -.--.
static const int pattern_rparen[] = {1, 0, 1, 1, 0, 1, -1};   // -.--.-
static const int pattern_ampersand[] = {0, 1, 0, 0, 0, -1};   // .-...
static const int pattern_colon[] = {1, 1, 1, 0, 0, 0, -1};    // ---...
static const int pattern_semicolon[] = {1, 0, 1, 0, 1, 0, -1}; // -.-.-.
static const int pattern_equals[] = {1, 0, 0, 0, 1, -1};      // -...-
static const int pattern_plus[] = {0, 1, 0, 1, 0, -1};        // .-.-.
static const int pattern_hyphen[] = {1, 0, 0, 0, 0, 1, -1};   // -....-
static const int pattern_underscore[] = {0, 0, 1, 1, 0, 1, -1}; // ..--.-
static const int pattern_dquote[] = {0, 1, 0, 0, 1, 0, -1};   // .-..-.
static const int pattern_dollar[] = {0, 0, 0, 1, 0, 0, 1, -1}; // ...-..-
static const int pattern_at[] = {0, 1, 1, 0, 1, 0, -1};       // .--.-.

// Direct lookup table - fastest possible O(1) access
static const int* morse_patterns[256] = {
  // Uppercase letters
  ['A'] = pattern_A, ['B'] = pattern_B, ['C'] = pattern_C, ['D'] = pattern_D,
  ['E'] = pattern_E, ['F'] = pattern_F, ['G'] = pattern_G, ['H'] = pattern_H,
  ['I'] = pattern_I, ['J'] = pattern_J, ['K'] = pattern_K, ['L'] = pattern_L,
  ['M'] = pattern_M, ['N'] = pattern_N, ['O'] = pattern_O, ['P'] = pattern_P,
  ['Q'] = pattern_Q, ['R'] = pattern_R, ['S'] = pattern_S, ['T'] = pattern_T,
  ['U'] = pattern_U, ['V'] = pattern_V, ['W'] = pattern_W, ['X'] = pattern_X,
  ['Y'] = pattern_Y, ['Z'] = pattern_Z,

  // Lowercase letters (same patterns as uppercase)
  ['a'] = pattern_A, ['b'] = pattern_B, ['c'] = pattern_C, ['d'] = pattern_D,
  ['e'] = pattern_E, ['f'] = pattern_F, ['g'] = pattern_G, ['h'] = pattern_H,
  ['i'] = pattern_I, ['j'] = pattern_J, ['k'] = pattern_K, ['l'] = pattern_L,
  ['m'] = pattern_M, ['n'] = pattern_N, ['o'] = pattern_O, ['p'] = pattern_P,
  ['q'] = pattern_Q, ['r'] = pattern_R, ['s'] = pattern_S, ['t'] = pattern_T,
  ['u'] = pattern_U, ['v'] = pattern_V, ['w'] = pattern_W, ['x'] = pattern_X,
  ['y'] = pattern_Y, ['z'] = pattern_Z,

  // Numbers
  ['0'] = pattern_0, ['1'] = pattern_1, ['2'] = pattern_2, ['3'] = pattern_3,
  ['4'] = pattern_4, ['5'] = pattern_5, ['6'] = pattern_6, ['7'] = pattern_7,
  ['8'] = pattern_8, ['9'] = pattern_9,

  // Punctuation
  ['.'] = pattern_period, [','] = pattern_comma, ['?'] = pattern_question,
  ['\''] = pattern_quote, ['!'] = pattern_exclaim, ['/'] = pattern_slash,
  ['('] = pattern_lparen, [')'] = pattern_rparen, ['&'] = pattern_ampersand,
  [':'] = pattern_colon, [';'] = pattern_semicolon, ['='] = pattern_equals,
  ['+'] = pattern_plus, ['-'] = pattern_hyphen, ['_'] = pattern_underscore,
  ['"'] = pattern_dquote, ['$'] = pattern_dollar, ['@'] = pattern_at
};

// Internal function for processing morse text - shared by timing and size functions
static size_t morse_timing_process(const char *text, const MorseTimingParams *params, MorseElement *out_elements, size_t max_elements) {
  if(!text || !params) return 0;
  if(params->wpm <= 0) return 0; // Invalid WPM

  // Initialize random seed if humanization is enabled
  if (params->humanization_factor > 0.0f) {
    unsigned int seed = params->random_seed;
    if (seed == 0) {
      // Use time-based seed for true randomness
      seed = (unsigned int)time(NULL);
    }
    srand(seed);
  }

  float dot_sec = DOT_LENGTH_WPM / params->wpm;
  size_t count = 0;
  size_t i = 0;

  while(text[i]) {
    // For size-only mode, continue processing even when out_elements is NULL
    // For timing mode, stop when buffer is full
    if(out_elements && count >= max_elements) break;
    
    char ch = text[i];

    // Handle spaces as inter-word gaps
    if(ch == ' ') {
      // Add inter-word gap (7 dot durations * word_gap_multiplier)
      float word_gap_duration = dot_sec * DOTS_PER_WORD_GAP * params->word_gap_multiplier;
      word_gap_duration = apply_humanization(word_gap_duration, params->humanization_factor);

      if(out_elements) {
        out_elements[count] = (MorseElement){MORSE_GAP, word_gap_duration};
      }
      count++;
      i++;
      continue;
    }

    // Handle prosigns in brackets [...]
    if(ch == '[') {
      i++; // Skip opening bracket

      // Process characters inside brackets (skip spaces and invalid chars)
      int prosign_char_count = 0;
      while(text[i] && text[i] != ']') {
        char prosign_ch = text[i];

        // Skip spaces inside prosigns
        if(prosign_ch == ' ') {
          i++;
          continue;
        }

        const int* pattern = morse_patterns[(unsigned char)prosign_ch];

        if(pattern) {
          // Add 1-dot gap between characters in prosign (except for first character)
          if(prosign_char_count > 0) {
            float prosign_gap_duration = apply_humanization(dot_sec, params->humanization_factor);
            if(out_elements) {
              if(count >= max_elements) break;
              out_elements[count] = (MorseElement){MORSE_GAP, prosign_gap_duration};
            }
            count++;
          }

          // Add pattern elements
          for(int j = 0; pattern[j] != -1; j++) {
            if(out_elements && count >= max_elements) break;
            
            MorseElementType type = (pattern[j] == 0) ? MORSE_DOT : MORSE_DASH;
            float base_duration = (type == MORSE_DOT) ? dot_sec : dot_sec * DOTS_PER_DASH;
            float duration = apply_humanization(base_duration, params->humanization_factor);

            if(out_elements) {
              out_elements[count] = (MorseElement){type, duration};
            }
            count++;

            // Add inter-element gap (except after last element)
            if(pattern[j+1] != -1) {
              float gap_duration = apply_humanization(dot_sec, params->humanization_factor);
              if(out_elements) {
                if(count >= max_elements) break;
                out_elements[count] = (MorseElement){MORSE_GAP, gap_duration};
              }
              count++;
            }
          }
          prosign_char_count++;
        }
        i++;
      }

      // Skip closing bracket
      if(text[i] == ']') {
        i++;
      }

    } else {
      // Handle regular character
      const int* pattern = morse_patterns[(unsigned char)ch];

      if(pattern) {
        // Add inter-character gap if not the first character
        if(count > 0) {
          // Check if last element was not already a gap to avoid duplicate gaps
          int should_add_gap = 1;
          if(out_elements && count > 0) {
            should_add_gap = (out_elements[count-1].type != MORSE_GAP);
          }
          
          if(should_add_gap) {
            float inter_char_duration = apply_humanization(dot_sec * DOTS_PER_CHAR_GAP, params->humanization_factor);
            if(out_elements) {
              if(count >= max_elements) break;
              out_elements[count] = (MorseElement){MORSE_GAP, inter_char_duration};
            }
            count++;
          }
        }

        // Add pattern elements
        for(int j = 0; pattern[j] != -1; j++) {
          if(out_elements && count >= max_elements) break;
          
          MorseElementType type = (pattern[j] == 0) ? MORSE_DOT : MORSE_DASH;
          float base_duration = (type == MORSE_DOT) ? dot_sec : dot_sec * DOTS_PER_DASH;
          float duration = apply_humanization(base_duration, params->humanization_factor);

          if(out_elements) {
            out_elements[count] = (MorseElement){type, duration};
          }
          count++;

          // Add inter-element gap (except after last element)
          if(pattern[j+1] != -1) {
            float gap_duration = apply_humanization(dot_sec, params->humanization_factor);
            if(out_elements) {
              if(count >= max_elements) break;
              out_elements[count] = (MorseElement){MORSE_GAP, gap_duration};
            }
            count++;
          }
        }
      }
      i++;
    }
  }

  return count;
}

size_t morse_timing(MorseElement *out_elements, size_t max_elements, const char *text, const MorseTimingParams *params) {
  if(!out_elements) return 0;
  return morse_timing_process(text, params, out_elements, max_elements);
}

// Radio mode audio generation
static size_t morse_audio_radio(const MorseElement *events, size_t element_count, float *out_buffer, size_t max_samples, const MorseAudioParams *params) {
  const MorseRadioParams *radio = &params->mode_params.radio;
  if(radio->freq_hz <= 0.0f || radio->freq_hz > 20000.0f) return 0; // Invalid frequency

  float clamped_volume = params->volume < 0.0f ? 0.0f : (params->volume > 1.0f ? 1.0f : params->volume);

  size_t samples_written = 0;
  for(size_t i = 0; i < element_count && samples_written < max_samples; i++) {
    const MorseElement *elem = &events[i];
    size_t elem_samples = (size_t)(elem->duration_seconds * params->sample_rate);

    if(elem->type == MORSE_GAP) {
      for(size_t j = 0; j < elem_samples && samples_written < max_samples; j++) {
        float signal = 0.0f;

        // Add background static if enabled (continuous during gaps)
        if (radio->background_static_level > 0.0f) {
          signal = generate_white_noise() * radio->background_static_level * clamped_volume;
        }

        out_buffer[samples_written++] = signal;
      }
    } else {
      size_t attack_samples = (size_t)((ATTACK_MS / 1000.0f) * params->sample_rate);
      size_t release_samples = (size_t)((RELEASE_MS / 1000.0f) * params->sample_rate);

      // Clamp envelope lengths to element duration
      if(attack_samples > elem_samples / 2) attack_samples = elem_samples / 2;
      if(release_samples > elem_samples / 2) release_samples = elem_samples / 2;

      size_t sustain_start = attack_samples;
      size_t release_start = elem_samples - release_samples;

      // Attack phase
      for(size_t j = 0; j < attack_samples && samples_written < max_samples; j++) {
        float t = (float)j / params->sample_rate;
        float envelope = (float)j / attack_samples;
        float waveform = generate_waveform(radio->waveform_type, radio->freq_hz, t);
        float signal = waveform * clamped_volume * envelope;

        // Add background static if enabled
        if (radio->background_static_level > 0.0f) {
          signal += generate_white_noise() * radio->background_static_level * clamped_volume;
        }

        out_buffer[samples_written++] = signal;
      }

      // Sustain phase
      for(size_t j = sustain_start; j < release_start && samples_written < max_samples; j++) {
        float t = (float)j / params->sample_rate;
        float waveform = generate_waveform(radio->waveform_type, radio->freq_hz, t);
        float signal = waveform * clamped_volume;

        // Add background static if enabled
        if (radio->background_static_level > 0.0f) {
          signal += generate_white_noise() * radio->background_static_level * clamped_volume;
        }

        out_buffer[samples_written++] = signal;
      }

      // Release phase
      for(size_t j = release_start; j < elem_samples && samples_written < max_samples; j++) {
        float t = (float)j / params->sample_rate;
        float envelope = (float)(elem_samples - j) / release_samples;
        float waveform = generate_waveform(radio->waveform_type, radio->freq_hz, t);
        float signal = waveform * clamped_volume * envelope;

        // Add background static if enabled
        if (radio->background_static_level > 0.0f) {
          signal += generate_white_noise() * radio->background_static_level * clamped_volume;
        }

        out_buffer[samples_written++] = signal;
      }
    }
  }
  return samples_written;
}

// Telegraph mode audio generation
static size_t morse_audio_telegraph(const MorseElement *events, size_t element_count, float *out_buffer, size_t max_samples, const MorseAudioParams *params) {
  const MorseTelegraphParams *telegraph = &params->mode_params.telegraph;
  float clamped_volume = params->volume < 0.0f ? 0.0f : (params->volume > 1.0f ? 1.0f : params->volume);

  size_t samples_written = 0;
  for(size_t i = 0; i < element_count && samples_written < max_samples; i++) {
    const MorseElement *elem = &events[i];
    size_t elem_samples = (size_t)(elem->duration_seconds * params->sample_rate);

    if(elem->type == MORSE_GAP) {
      // Silence for gaps
      for(size_t j = 0; j < elem_samples && samples_written < max_samples; j++) {
        out_buffer[samples_written++] = 0.0f;
      }
    } else {
      // Generate click at start of dot/dash
      size_t click_samples = (size_t)(TELEGRAPH_CLICK_DURATION_SEC * params->sample_rate);
      if(click_samples > elem_samples) click_samples = elem_samples;

      for(size_t j = 0; j < click_samples && samples_written < max_samples; j++) {
        float t = (float)j / params->sample_rate;

        // Sharp attack followed by resonant decay
        // Map 0-1 sharpness to exponential range
        float sharpness_factor = TELEGRAPH_MIN_SHARPNESS +
                                telegraph->click_sharpness * (TELEGRAPH_MAX_SHARPNESS - TELEGRAPH_MIN_SHARPNESS);
        float attack_envelope = expf(-t * sharpness_factor);
        float resonance = sinf(2.0f * M_PI * telegraph->resonance_freq * t);
        float decay = expf(-t * telegraph->decay_rate);

        out_buffer[samples_written++] = resonance * attack_envelope * decay * clamped_volume;
      }

      // Fill remainder with silence
      for(size_t j = click_samples; j < elem_samples && samples_written < max_samples; j++) {
        out_buffer[samples_written++] = 0.0f;
      }
    }
  }
  return samples_written;
}

// Main audio generation function - dispatches by mode
size_t morse_audio(const MorseElement *events, size_t element_count, float *out_buffer, size_t max_samples, const MorseAudioParams *params) {
  if(!events || !out_buffer || !params) return 0;
  if(params->sample_rate <= 0 || params->sample_rate > 192000) return 0;

  switch(params->audio_mode) {
    case MORSE_RADIO:
      return morse_audio_radio(events, element_count, out_buffer, max_samples, params);
    case MORSE_TELEGRAPH:
      return morse_audio_telegraph(events, element_count, out_buffer, max_samples, params);
    default:
      return 0; // Unknown mode
  }
}

size_t morse_timing_size(const char *text, const MorseTimingParams *params) {
  return morse_timing_process(text, params, NULL, 0);
}

size_t morse_audio_size(const MorseElement *events, size_t element_count, const MorseAudioParams *params) {
  if(!events || !params) return 0;
  if(params->sample_rate <= 0 || params->sample_rate > 192000) return 0; // Invalid sample rate

  size_t total_samples = 0;
  for(size_t i = 0; i < element_count; i++) {
    const MorseElement *elem = &events[i];
    size_t elem_samples = (size_t)(elem->duration_seconds * params->sample_rate);
    total_samples += elem_samples;
  }
  return total_samples;
}

