// Import the WebAssembly module
import MorseWasmModule from './morse-wasm.js';

// Module loading
let module = null;

// Optional promise for users who want to ensure module is loaded
export const ready = MorseWasmModule({
  locateFile: name => new URL(name, import.meta.url).href
}).then(loadedModule => {
  module = loadedModule;
  return loadedModule;
});

// Option keys enum mirror for convenience
export const OPT = {
  WPM: 0,
  SAMPLE_RATE: 1,
  FREQ_HZ: 2,
  VOLUME: 3,
  WORD_GAP_MULTIPLIER: 4,
  HUMANIZATION_FACTOR: 5,
  RANDOM_SEED: 6
};

/**
 * @typedef {Object} MorseTimingParams
 * @property {string} text - Text to convert to Morse code
 * @property {number} [wpm=20] - Words per minute
 * @property {number} [wordGapMultiplier=1.0] - Word gap scaling factor
 * @property {number} [humanizationFactor=0.0] - Timing randomization factor (0.0-1.0)
 * @property {number} [randomSeed=0] - Random seed for reproducible humanization (0 = use time)
 */

/**
 * @typedef {Object} MorseAudioParams
 * @property {string} text - Text to convert to Morse code
 * @property {number} [wpm=20] - Words per minute
 * @property {number} [sampleRate=22050] - Audio sample rate
 * @property {number} [frequency=440] - Tone frequency in Hz
 * @property {number} [volume=0.5] - Audio volume (0.0 to 1.0)
 * @property {number} [wordGapMultiplier=1.0] - Word gap scaling factor
 * @property {number} [humanizationFactor=0.0] - Timing randomization factor (0.0-1.0)
 * @property {number} [randomSeed=0] - Random seed for reproducible humanization (0 = use time)
 */

/**
 * Validates timing parameters
 * @param {MorseTimingParams} params - Parameters to validate
 * @throws {Error} If validation fails
 */
function validateTimingParams({ text, wpm, wordGapMultiplier, humanizationFactor, randomSeed }) {
  if (!text || typeof text !== 'string') throw new Error("Invalid text input");
  if (!Number.isInteger(wpm) || wpm <= 0) throw new Error("WPM must be a positive integer");
  if (typeof wordGapMultiplier !== 'number' || wordGapMultiplier < 0) throw new Error("Word gap multiplier must be a non-negative number");
  if (typeof humanizationFactor !== 'number' || humanizationFactor < 0 || humanizationFactor > 1) throw new Error("Humanization factor must be between 0.0 and 1.0");
  if (!Number.isInteger(randomSeed) || randomSeed < 0) throw new Error("Random seed must be a non-negative integer");
}

/**
 * Validates audio parameters
 * @param {MorseAudioParams} params - Parameters to validate
 * @throws {Error} If validation fails
 */
function validateAudioParams({ text, wpm, sampleRate, frequency, volume, wordGapMultiplier, humanizationFactor, randomSeed }) {
  validateTimingParams({ text, wpm, wordGapMultiplier, humanizationFactor, randomSeed });
  if (!Number.isInteger(sampleRate) || sampleRate <= 0 || sampleRate > 192000) {
    throw new Error("Sample rate must be between 1 and 192000 Hz");
  }
  if (typeof frequency !== 'number' || frequency <= 0 || frequency > 20000) {
    throw new Error("Frequency must be between 1 and 20000 Hz");
  }
  if (typeof volume !== 'number' || volume < 0 || volume > 1) {
    throw new Error("Volume must be between 0.0 and 1.0");
  }
}

/**
 * Generates Morse code timing elements from text
 * @param {MorseTimingParams} params - Timing generation parameters
 * @returns {Array} Array of timing elements
 * @throws {Error} If WebAssembly module not loaded or generation failed
 */
function generateMorseTiming({
  text,
  wpm = 20,
  wordGapMultiplier = 1.0,
  humanizationFactor = 0.0,
  randomSeed = 0
}) {
  if (!module) throw new Error("WebAssembly module not loaded yet. Try awaiting ready first.");
  validateTimingParams({ text, wpm, wordGapMultiplier, humanizationFactor, randomSeed });

  const morse_new = module.cwrap("morse_new", "number", []);
  const morse_free = module.cwrap("morse_free", "void", ["number"]);
  const morse_set_i32 = module.cwrap("morse_set_i32", "number", ["number", "number", "number"]);
  const morse_set_f32 = module.cwrap("morse_set_f32", "number", ["number", "number", "number"]);
  const morse_timing_size_ctx = module.cwrap("morse_timing_size_ctx", "number", ["number", "string"]);
  const morse_timing_fill_ctx = module.cwrap("morse_timing_fill_ctx", "number",
    ["number", "string", "number", "number", "number"]);

  // Create context and set parameters
  const ctx = morse_new();
  if (!ctx) throw new Error("Failed to create Morse context");

  morse_set_i32(ctx, OPT.WPM, wpm);
  morse_set_f32(ctx, OPT.WORD_GAP_MULTIPLIER, wordGapMultiplier);
  morse_set_f32(ctx, OPT.HUMANIZATION_FACTOR, humanizationFactor);
  morse_set_i32(ctx, OPT.RANDOM_SEED, randomSeed);

  // Get exact size needed
  const elementCount = morse_timing_size_ctx(ctx, text);
  if (elementCount === 0) {
    morse_free(ctx);
    throw new Error("Failed to generate Morse timing");
  }

  // Allocate arrays for results
  const typesPtr = module._malloc(elementCount * 4); // int array
  const dursPtr = module._malloc(elementCount * 4);  // float array

  // Fill arrays
  const actualCount = morse_timing_fill_ctx(ctx, text, typesPtr, dursPtr, elementCount);

  // Convert to JavaScript objects
  const elements = [];
  const buffer = module.HEAPU8.buffer;
  const typesView = new Int32Array(buffer, typesPtr, actualCount);
  const dursView = new Float32Array(buffer, dursPtr, actualCount);

  for (let i = 0; i < actualCount; i++) {
    const type = typesView[i];
    const duration = dursView[i];

    elements.push({
      type: type === 0 ? "dot" : type === 1 ? "dash" : "gap",
      duration_seconds: duration
    });
  }

  // Cleanup
  module._free(typesPtr);
  module._free(dursPtr);
  morse_free(ctx);

  return elements;
}

/**
 * Generates Morse code audio from text
 * @param {MorseAudioParams} params - Audio generation parameters
 * @returns {Object} Audio result
 * @throws {Error} If WebAssembly module not loaded or generation failed
 */
function generateMorseAudio({
  text,
  wpm = 20,
  sampleRate = 22050,
  frequency = 440,
  volume = 0.5,
  wordGapMultiplier = 1.0,
  humanizationFactor = 0.0,
  randomSeed = 0
}) {
  if (!module) throw new Error("WebAssembly module not loaded yet. Try awaiting ready first.");
  validateAudioParams({ text, wpm, sampleRate, frequency, volume, wordGapMultiplier, humanizationFactor, randomSeed });

  const morse_new = module.cwrap("morse_new", "number", []);
  const morse_free = module.cwrap("morse_free", "void", ["number"]);
  const morse_set_i32 = module.cwrap("morse_set_i32", "number", ["number", "number", "number"]);
  const morse_set_f32 = module.cwrap("morse_set_f32", "number", ["number", "number", "number"]);
  const morse_timing_size_ctx = module.cwrap("morse_timing_size_ctx", "number", ["number", "string"]);
  const morse_timing_fill_ctx = module.cwrap("morse_timing_fill_ctx", "number", 
    ["number", "string", "number", "number", "number"]);
  const morse_audio_size_ctx = module.cwrap("morse_audio_size_ctx", "number", 
    ["number", "number", "number", "number"]);
  const morse_audio_fill_ctx = module.cwrap("morse_audio_fill_ctx", "number", 
    ["number", "number", "number", "number", "number", "number"]);

  // Create context and set all parameters
  const ctx = morse_new();
  if (!ctx) throw new Error("Failed to create Morse context");

  morse_set_i32(ctx, OPT.WPM, wpm);
  morse_set_i32(ctx, OPT.SAMPLE_RATE, sampleRate);
  morse_set_f32(ctx, OPT.FREQ_HZ, frequency);
  morse_set_f32(ctx, OPT.VOLUME, volume);
  morse_set_f32(ctx, OPT.WORD_GAP_MULTIPLIER, wordGapMultiplier);
  morse_set_f32(ctx, OPT.HUMANIZATION_FACTOR, humanizationFactor);
  morse_set_i32(ctx, OPT.RANDOM_SEED, randomSeed);

  // Get timing elements first
  const elementCount = morse_timing_size_ctx(ctx, text);
  if (elementCount === 0) {
    morse_free(ctx);
    throw new Error("Failed to generate Morse timing");
  }

  const typesPtr = module._malloc(elementCount * 4); // int array
  const dursPtr = module._malloc(elementCount * 4);  // float array

  const actualCount = morse_timing_fill_ctx(ctx, text, typesPtr, dursPtr, elementCount);

  // Calculate total duration
  let totalDuration = 0;
  const buffer = module.HEAPU8.buffer;
  const dursView = new Float32Array(buffer, dursPtr, actualCount);
  for (let i = 0; i < actualCount; i++) {
    totalDuration += dursView[i];
  }

  // Get audio size and fill
  const audioSize = morse_audio_size_ctx(ctx, typesPtr, dursPtr, actualCount);
  const samplesPtr = module._malloc(audioSize * 4); // float array

  const samplesGenerated = morse_audio_fill_ctx(ctx, typesPtr, dursPtr, actualCount, samplesPtr, audioSize);

  // Copy audio data to JavaScript array
  const audioData = new Float32Array(samplesGenerated);
  const samplesView = new Float32Array(buffer, samplesPtr, samplesGenerated);
  for (let i = 0; i < samplesGenerated; i++) {
    audioData[i] = samplesView[i];
  }

  // Cleanup
  module._free(typesPtr);
  module._free(dursPtr);
  module._free(samplesPtr);
  morse_free(ctx);

  return {
    audioData,
    sampleRate,
    duration: totalDuration
  };
}

/**
 * Plays audio result using Web Audio API
 * @param {Object} audioResult - Result from generateMorseAudio
 * @returns {AudioBufferSourceNode} Audio source node
 * @throws {Error} If audioResult is invalid or Web Audio API fails
 */
function playMorseAudio(audioResult) {
  if (!audioResult) throw new Error("No audio result provided");
  if (!audioResult.audioData || audioResult.audioData.length === 0) {
    throw new Error("Invalid audio data");
  }
  if (!audioResult.sampleRate || audioResult.sampleRate <= 0) {
    throw new Error("Invalid sample rate");
  }

  try {
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    if (!AudioContextClass) {
      throw new Error("Web Audio API not supported in this browser");
    }

    const audioContext = new AudioContextClass();
    const buffer = audioContext.createBuffer(1, audioResult.audioData.length, audioResult.sampleRate);
    buffer.copyToChannel(audioResult.audioData, 0);

    const source = audioContext.createBufferSource();
    source.buffer = buffer;
    source.connect(audioContext.destination);
    source.start();

    return source;
  } catch (error) {
    throw new Error(`Failed to play audio: ${error.message}`);
  }
}

// Export individual functions for tree shaking
export { generateMorseTiming, generateMorseAudio, playMorseAudio };