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
  VOLUME: 3
};

/**
 * Generates Morse code timing elements from text
 * @param {Object} params - Timing generation parameters
 * @param {string} params.text - Text to convert to Morse code
 * @param {number} [params.wpm=20] - Words per minute
 * @returns {Array} Array of timing elements
 * @throws {Error} If WebAssembly module not loaded or generation failed
 */
function generateMorseTiming({
  text,
  wpm = 20
}) {
  if (!module) throw new Error("WebAssembly module not loaded yet. Try awaiting ready first.");

  const morse_new = module.cwrap("morse_new", "number", []);
  const morse_free = module.cwrap("morse_free", "void", ["number"]);
  const morse_set_i32 = module.cwrap("morse_set_i32", "number", ["number", "number", "number"]);
  const morse_timing_size_ctx = module.cwrap("morse_timing_size_ctx", "number", ["number", "string"]);
  const morse_timing_fill_ctx = module.cwrap("morse_timing_fill_ctx", "number", 
    ["number", "string", "number", "number", "number"]);

  // Create context and set WPM
  const ctx = morse_new();
  if (!ctx) throw new Error("Failed to create Morse context");

  morse_set_i32(ctx, OPT.WPM, wpm);

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
  for (let i = 0; i < actualCount; i++) {
    const type = module.HEAP32[(typesPtr >> 2) + i];
    const duration = module.HEAPF32[(dursPtr >> 2) + i];

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
 * @param {Object} params - Audio generation parameters
 * @param {string} params.text - Text to convert to Morse code
 * @param {number} [params.wpm=20] - Words per minute
 * @param {number} [params.sampleRate=22050] - Audio sample rate
 * @param {number} [params.frequency=440] - Tone frequency in Hz
 * @param {number} [params.volume=0.5] - Audio volume (0.0 to 1.0)
 * @returns {Object} Audio result
 * @throws {Error} If WebAssembly module not loaded or generation failed
 */
function generateMorseAudio({
  text,
  wpm = 20,
  sampleRate = 22050,
  frequency = 440,
  volume = 0.5
}) {
  if (!module) throw new Error("WebAssembly module not loaded yet. Try awaiting ready first.");

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
  for (let i = 0; i < actualCount; i++) {
    totalDuration += module.HEAPF32[(dursPtr >> 2) + i];
  }

  // Get audio size and fill
  const audioSize = morse_audio_size_ctx(ctx, typesPtr, dursPtr, actualCount);
  const samplesPtr = module._malloc(audioSize * 4); // float array

  const samplesGenerated = morse_audio_fill_ctx(ctx, typesPtr, dursPtr, actualCount, samplesPtr, audioSize);

  // Copy audio data to JavaScript array
  const audioData = new Float32Array(samplesGenerated);
  for (let i = 0; i < samplesGenerated; i++) {
    audioData[i] = module.HEAPF32[(samplesPtr >> 2) + i];
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
 * @throws {Error} If audioResult is invalid
 */
function playMorseAudio(audioResult) {
  if (!audioResult) throw new Error("No audio result provided");

  const audioContext = new (window.AudioContext || window.webkitAudioContext)();
  const buffer = audioContext.createBuffer(1, audioResult.audioData.length, audioResult.sampleRate);
  buffer.copyToChannel(audioResult.audioData, 0);

  const source = audioContext.createBufferSource();
  source.buffer = buffer;
  source.connect(audioContext.destination);
  source.start();

  return source;
}

// Export individual functions for tree shaking
export { generateMorseTiming, generateMorseAudio, playMorseAudio };