#include "morse.h"
#include <math.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct {
  float *data;
  int *assignments;
  float *centroids;
  int n_points;
  int k;
} KMeansCluster;
static KMeansCluster* kmeans_create(float *data, int n_points, int k) {
  if (!data || n_points <= 0 || k <= 0 || k > n_points) return NULL;

  KMeansCluster *cluster = malloc(sizeof(KMeansCluster));
  if (!cluster) return NULL;

  cluster->data = data;
  cluster->n_points = n_points;
  cluster->k = k;

  cluster->assignments = calloc(n_points, sizeof(int));
  cluster->centroids = malloc(k * sizeof(float));

  if (!cluster->assignments || !cluster->centroids) {
    free(cluster->assignments);
    free(cluster->centroids);
    free(cluster);
    return NULL;
  }

  float min_val = data[0], max_val = data[0];
  for (int i = 1; i < n_points; i++) {
    if (data[i] < min_val) min_val = data[i];
    if (data[i] > max_val) max_val = data[i];
  }

  for (int i = 0; i < k; i++) {
    cluster->centroids[i] = min_val + (max_val - min_val) * i / (k - 1);
  }

  return cluster;
}

static void kmeans_free(KMeansCluster *cluster) {
  if (!cluster) return;
  free(cluster->assignments);
  free(cluster->centroids);
  free(cluster);
}
static float kmeans_iterate(KMeansCluster *cluster) {
  if (!cluster) return 0.0f;

  for (int i = 0; i < cluster->n_points; i++) {
    float min_dist = fabsf(cluster->data[i] - cluster->centroids[0]);
    int best_cluster = 0;

    for (int j = 1; j < cluster->k; j++) {
      float dist = fabsf(cluster->data[i] - cluster->centroids[j]);
      if (dist < min_dist) {
        min_dist = dist;
        best_cluster = j;
      }
    }
    cluster->assignments[i] = best_cluster;
  }
  float total_movement = 0.0f;
  for (int j = 0; j < cluster->k; j++) {
    float sum = 0.0f;
    int count = 0;

    for (int i = 0; i < cluster->n_points; i++) {
      if (cluster->assignments[i] == j) {
        sum += cluster->data[i];
        count++;
      }
    }

    float new_centroid = (count > 0) ? sum / count : cluster->centroids[j];
    total_movement += fabsf(new_centroid - cluster->centroids[j]);
    cluster->centroids[j] = new_centroid;
  }

  return total_movement;
}

static int kmeans_cluster(float *data, int n_points, int k, int *assignments, float *centroids, const MorseInterpretParams *params) {
  KMeansCluster *cluster = kmeans_create(data, n_points, k);
  if (!cluster) return 0;

  for (int iter = 0; iter < params->max_k_means_iterations; iter++) {
    float movement = kmeans_iterate(cluster);
    if (movement < params->convergence_threshold) {
      break;
    }
  }

  memcpy(assignments, cluster->assignments, n_points * sizeof(int));
  memcpy(centroids, cluster->centroids, k * sizeof(float));
  for (int i = 0; i < k - 1; i++) {
    for (int j = 0; j < k - 1 - i; j++) {
      if (centroids[j] > centroids[j + 1]) {
        float temp = centroids[j];
        centroids[j] = centroids[j + 1];
        centroids[j + 1] = temp;

        for (int p = 0; p < n_points; p++) {
          if (assignments[p] == j) assignments[p] = j + 1;
          else if (assignments[p] == j + 1) assignments[p] = j;
        }
      }
    }
  }

  kmeans_free(cluster);
  return 1;
}

static char pattern_to_char(const int *pattern, int length) {
  extern const int* morse_patterns[256];

  for (int ch = 0; ch < 256; ch++) {
    const int* stored_pattern = morse_patterns[ch];
    if (!stored_pattern) continue;

    int stored_length = 0;
    while (stored_pattern[stored_length] != -1) stored_length++;

    if (stored_length != length) continue;

    int match = 1;
    for (int i = 0; i < length; i++) {
      if (pattern[i] != stored_pattern[i]) {
        match = 0;
        break;
      }
    }

    if (match) return (char)ch;
  }

  return '?';
}

// Convert MorseElements to MorseSignals (utility function for testing)
size_t morse_elements_to_signals(const MorseElement *elements, size_t element_count, MorseSignal *out_signals, size_t max_signals) {
  if (!elements || !out_signals || element_count == 0) return 0;

  size_t signal_count = 0;

  for (size_t i = 0; i < element_count && signal_count < max_signals; i++) {
    const MorseElement *elem = &elements[i];

    out_signals[signal_count] = (MorseSignal){
      .on = (elem->type != MORSE_GAP),
      .seconds = elem->duration_seconds
    };
    signal_count++;
  }

  return signal_count;
}

size_t morse_interpret_text_size(const MorseSignal *signals, size_t signal_count, const MorseInterpretParams *params) {
  if (!signals || signal_count == 0 || !params) return 0;

  size_t estimated_size = signal_count + 100;
  if (estimated_size > (size_t)params->max_output_length) {
    estimated_size = params->max_output_length;
  }

  return estimated_size;
}

void morse_interpret_result_free(MorseInterpretResult *result) {
  if (!result) return;
  free(result->text);
  result->text = NULL;
  result->text_length = 0;
}
MorseInterpretResult morse_interpret(const MorseSignal *signals, size_t signal_count, const MorseInterpretParams *params) {
  MorseInterpretResult result = {0};

  if (!signals || signal_count == 0 || !params) {
    return result;
  }

  float *on_durations = malloc(signal_count * sizeof(float));
  float *off_durations = malloc(signal_count * sizeof(float));
  int on_count = 0, off_count = 0;

  if (!on_durations || !off_durations) {
    free(on_durations);
    free(off_durations);
    return result;
  }

  int start_index = 0;
  if (signal_count > 0 && !signals[0].on) {
    start_index = 1;
  }

  int end_index = signal_count;
  if (signal_count > 0 && !signals[signal_count - 1].on) {
    end_index = signal_count - 1;
  }

  for (size_t i = start_index; i < end_index; i++) {
    if (signals[i].seconds >= params->noise_threshold) {
      if (signals[i].on) {
        on_durations[on_count++] = signals[i].seconds;
      } else {
        off_durations[off_count++] = signals[i].seconds;
      }
    }
  }
  if (on_count == 0) {
    free(on_durations);
    free(off_durations);
    return result;
  }

  int *on_assignments = malloc(on_count * sizeof(int));
  float on_centroids[2] = {0};

  if (!on_assignments) {
    free(on_durations);
    free(off_durations);
    return result;
  }

  if (on_count == 1) {
    on_assignments[0] = 0;
    on_centroids[0] = on_durations[0];
    on_centroids[1] = on_durations[0] * 3.0f;
  } else {
    if (!kmeans_cluster(on_durations, on_count, 2, on_assignments, on_centroids, params)) {
      free(on_durations);
      free(off_durations);
      free(on_assignments);
      return result;
    }
  }

  int *off_assignments = NULL;
  float off_centroids[3] = {0};
  int off_clusters = 0;

  if (off_count > 0) {
    off_assignments = malloc(off_count * sizeof(int));

    if (off_count < 3) {
      off_clusters = off_count;
      if (off_count > 0) {
        if (!kmeans_cluster(off_durations, off_count, off_clusters, off_assignments, off_centroids, params)) {
            off_clusters = 0;
        }
      }
    } else {
      off_clusters = 3;
      if (!kmeans_cluster(off_durations, off_count, off_clusters, off_assignments, off_centroids, params)) {
          off_clusters = 0;
      }
    }
  }

    size_t text_size = morse_interpret_text_size(signals, signal_count, params);
    result.text = calloc(text_size, sizeof(char));

    if (!result.text) {
        free(on_durations);
        free(off_durations);
        free(on_assignments);
        free(off_assignments);
        return result;
    }

    int effective_off_clusters = off_clusters;
    if (off_clusters == 3) {
        float ratio1 = off_centroids[1] / off_centroids[0];
        float ratio2 = off_centroids[2] / off_centroids[1];
        const float MERGE_THRESHOLD = 1.9f;

        if (ratio2 < MERGE_THRESHOLD) {
            effective_off_clusters = 2;
            for (int i = 0; i < off_count; i++) {
                if (off_assignments[i] == 2) off_assignments[i] = 1;
            }
        }

        if (ratio1 < MERGE_THRESHOLD) {
            effective_off_clusters = (effective_off_clusters == 2) ? 1 : 2;
            for (int i = 0; i < off_count; i++) {
                if (off_assignments[i] == 1) off_assignments[i] = 0;
            }
        }
    }

  int on_idx = 0, off_idx = 0;
  int current_pattern[10];
  int pattern_length = 0;
  size_t text_pos = 0;
  int patterns_recognized = 0;
  int signals_processed = 0;

  for (size_t i = 0; i < signal_count && text_pos < text_size - 1; i++) {
    const MorseSignal *sig = &signals[i];

    if (sig->seconds < params->noise_threshold) {
      continue;
    }

    signals_processed++;

    if (sig->on) {
      if (on_idx < on_count) {
        current_pattern[pattern_length++] = on_assignments[on_idx];
        on_idx++;
      }
    } else {
      int gap_type = 0;

      if (off_idx < off_count && off_assignments && effective_off_clusters > 0) {
        gap_type = off_assignments[off_idx];
        off_idx++;
      }

      if (pattern_length > 0) {
        if (effective_off_clusters >= 3 && gap_type == 2) {
          char ch = pattern_to_char(current_pattern, pattern_length);
          if (ch != '?') {
            result.text[text_pos++] = ch;
            patterns_recognized++;
          }
          result.text[text_pos++] = ' ';
          pattern_length = 0;
        } else if ((effective_off_clusters >= 2 && gap_type >= 1) ||
                   (effective_off_clusters == 1 && off_centroids[0] > on_centroids[0] * 2.0f)) {
          char ch = pattern_to_char(current_pattern, pattern_length);
          if (ch != '?') {
            result.text[text_pos++] = ch;
            patterns_recognized++;
          }
          pattern_length = 0;
        }
      }
    }

    if (pattern_length >= 9) {
      char ch = pattern_to_char(current_pattern, pattern_length);
      if (ch != '?') {
        result.text[text_pos++] = ch;
        patterns_recognized++;
      }
      pattern_length = 0;
    }
  }

  if (pattern_length > 0) {
    char ch = pattern_to_char(current_pattern, pattern_length);
    if (ch != '?') {
      result.text[text_pos++] = ch;
      patterns_recognized++;
    }
  }

  while (text_pos > 0 && result.text[text_pos - 1] == ' ') {
    text_pos--;
  }
  result.text[text_pos] = '\0';

  result.text_length = text_pos;
  result.signals_processed = signals_processed;
  result.patterns_recognized = patterns_recognized;

  if (signals_processed > 0) {
    result.confidence = (float)patterns_recognized / (signals_processed / 2.0f);
    if (result.confidence > 1.0f) result.confidence = 1.0f;
    if (result.confidence < 0.0f) result.confidence = 0.0f;
  }
  free(on_durations);
  free(off_durations);
  free(on_assignments);
  free(off_assignments);

  return result;
}