#ifndef INTERPRET_MORSE_H
#define INTERPRET_MORSE_H

typedef struct {
  bool on;
  float seconds;
} MorseSignal;

typedef struct {
  int max_k_means_iterations;
  float convergence_threshold;
  float noise_threshold;
  int max_output_length;
} MorseInterpretParams;

typedef struct {
  char *text;
  size_t text_length;
  float confidence;
  int signals_processed;
  int patterns_recognized;
} MorseInterpretResult;

#define MORSE_DEFAULT_INTERPRET_PARAMS (MorseInterpretParams){ \
  .max_k_means_iterations = 100, \
  .convergence_threshold = 0.001f, \
  .noise_threshold = 0.001f, \
  .max_output_length = 1000 \
}
MorseInterpretResult morse_interpret(const MorseSignal *signals, size_t signal_count, const MorseInterpretParams *params);
size_t morse_interpret_text_size(const MorseSignal *signals, size_t signal_count, const MorseInterpretParams *params);
void morse_interpret_result_free(MorseInterpretResult *result);

size_t morse_elements_to_signals(const MorseElement *elements, size_t element_count, MorseSignal *out_signals, size_t max_signals);

#endif