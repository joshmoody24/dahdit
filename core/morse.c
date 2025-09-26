#include "morse.h"
#include <math.h>

const float DOT_LENGTH_WPM = 1.2f;      // Standard ITU timing formula: dot duration = 1.2 / WPM seconds
const int DOTS_PER_DASH = 3;           // ITU specification: dash = 3 dot durations
const float ATTACK_MS = 5.0f;          // Envelope attack time to prevent audio clicks
const float RELEASE_MS = 5.0f;         // Envelope release time to prevent audio clicks

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
      // Add inter-word gap (7 dot durations)
      if(out_elements) {
        out_elements[count] = (MorseElement){MORSE_GAP, dot_sec * 7};
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
            if(out_elements && count < max_elements) {
              out_elements[count] = (MorseElement){MORSE_GAP, dot_sec};
            }
            count++;
          }

          // Add pattern elements
          for(int j = 0; pattern[j] != -1; j++) {
            if(out_elements && count >= max_elements) break;
            
            MorseElementType type = (pattern[j] == 0) ? MORSE_DOT : MORSE_DASH;
            float duration = (type == MORSE_DOT) ? dot_sec : dot_sec * DOTS_PER_DASH;
            
            if(out_elements) {
              out_elements[count] = (MorseElement){type, duration};
            }
            count++;

            // Add inter-element gap (except after last element)
            if(pattern[j+1] != -1) {
              if(out_elements && count < max_elements) {
                out_elements[count] = (MorseElement){MORSE_GAP, dot_sec};
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
            if(out_elements && count < max_elements) {
              out_elements[count] = (MorseElement){MORSE_GAP, dot_sec * 3};
            }
            count++;
          }
        }

        // Add pattern elements
        for(int j = 0; pattern[j] != -1; j++) {
          if(out_elements && count >= max_elements) break;
          
          MorseElementType type = (pattern[j] == 0) ? MORSE_DOT : MORSE_DASH;
          float duration = (type == MORSE_DOT) ? dot_sec : dot_sec * DOTS_PER_DASH;
          
          if(out_elements) {
            out_elements[count] = (MorseElement){type, duration};
          }
          count++;

          // Add inter-element gap (except after last element)
          if(pattern[j+1] != -1) {
            if(out_elements && count < max_elements) {
              out_elements[count] = (MorseElement){MORSE_GAP, dot_sec};
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

size_t morse_audio(const MorseElement *events, size_t element_count, float *out_buffer, size_t max_samples, const MorseAudioParams *params) {
  if(!events || !out_buffer || !params) return 0;
  if(params->sample_rate <= 0 || params->sample_rate > 192000) return 0; // Invalid sample rate
  if(params->freq_hz <= 0.0f || params->freq_hz > 20000.0f) return 0; // Invalid frequency

  float clamped_volume = params->volume < 0.0f ? 0.0f : (params->volume > 1.0f ? 1.0f : params->volume);

  size_t samples_written = 0;
  for(size_t i = 0; i < element_count && samples_written < max_samples; i++) {
    const MorseElement *elem = &events[i];
    size_t elem_samples = (size_t)(elem->duration_seconds * params->sample_rate);

    if(elem->type == MORSE_GAP) {
      for(size_t j = 0; j < elem_samples && samples_written < max_samples; j++) {
        out_buffer[samples_written++] = 0.0f;
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
        out_buffer[samples_written++] = sinf(2.0f * M_PI * params->freq_hz * t) * clamped_volume * envelope;
      }

      // Sustain phase
      for(size_t j = sustain_start; j < release_start && samples_written < max_samples; j++) {
        float t = (float)j / params->sample_rate;
        out_buffer[samples_written++] = sinf(2.0f * M_PI * params->freq_hz * t) * clamped_volume;
      }

      // Release phase
      for(size_t j = release_start; j < elem_samples && samples_written < max_samples; j++) {
        float t = (float)j / params->sample_rate;
        float envelope = (float)(elem_samples - j) / release_samples;
        out_buffer[samples_written++] = sinf(2.0f * M_PI * params->freq_hz * t) * clamped_volume * envelope;
      }
    }
  }
  return samples_written;
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

