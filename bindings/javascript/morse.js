// Import the WebAssembly module
import MorseWasmModule from './morse-wasm.js';

// Module loading
let module = null;

// Optional promise for users who want to ensure module is loaded
export const ready = MorseWasmModule().then(loadedModule => {
  module = loadedModule;
  return loadedModule;
});

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

  const morse_timing = module.cwrap("morse_timing", "number",
    ["number", "number", "string", "number"]);

  // Allocate timing params struct
  const timingParamsPtr = module._malloc(4); // int wpm
  module.HEAP32[timingParamsPtr >> 2] = wpm;

  // Calculate max elements needed (worst case: ~10 elements per character)
  const maxElements = text.length * 10;
  const elementsPtr = module._malloc(maxElements * 8); // MorseElement is 8 bytes (int + float)

  // Generate timing
  const elementCount = morse_timing(elementsPtr, maxElements, text, timingParamsPtr);

  if (elementCount === 0) {
    module._free(timingParamsPtr);
    module._free(elementsPtr);
    throw new Error("Failed to generate Morse timing");
  }

  // Convert C structs to JavaScript objects
  const elements = [];
  for (let i = 0; i < elementCount; i++) {
    const elemPtr = elementsPtr + i * 8;
    const type = module.HEAP32[elemPtr >> 2]; // MorseElementType (int)
    const duration = module.HEAPF32[(elemPtr + 4) >> 2]; // duration_seconds (float)

    elements.push({
      type: type === 0 ? "dot" : type === 1 ? "dash" : "gap",
      duration_seconds: duration
    });
  }

  // Cleanup
  module._free(timingParamsPtr);
  module._free(elementsPtr);

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

  const morse_timing = module.cwrap("morse_timing", "number",
    ["number", "number", "string", "number"]);
  const morse_audio = module.cwrap("morse_audio", "number",
    ["number", "number", "number", "number", "number"]);

  // Allocate timing params struct
  const timingParamsPtr = module._malloc(4); // int wpm
  module.HEAP32[timingParamsPtr >> 2] = wpm;

  // Calculate max elements needed (worst case: ~10 elements per character)
  const maxElements = text.length * 10;
  const elementsPtr = module._malloc(maxElements * 8); // MorseElement is 8 bytes (int + float)

  // Generate timing
  const elementCount = morse_timing(elementsPtr, maxElements, text, timingParamsPtr);

  if (elementCount === 0) {
    module._free(timingParamsPtr);
    module._free(elementsPtr);
    throw new Error("Failed to generate Morse timing");
  }

  // Calculate total duration from actual elements
  let totalDuration = 0;
  for (let i = 0; i < elementCount; i++) {
    const elemPtr = elementsPtr + i * 8;
    const duration = module.HEAPF32[(elemPtr + 4) >> 2]; // duration_seconds is at offset 4
    totalDuration += duration;
  }

  // Allocate audio buffer based on actual duration
  const maxSamples = Math.ceil(totalDuration * sampleRate);
  const audioBufferPtr = module._malloc(maxSamples * 4); // float array

  // Allocate audio params struct
  const audioParamsPtr = module._malloc(12); // int + float + float
  module.HEAP32[audioParamsPtr >> 2] = sampleRate;
  module.HEAPF32[(audioParamsPtr + 4) >> 2] = frequency;
  module.HEAPF32[(audioParamsPtr + 8) >> 2] = volume;

  // Generate audio
  const samplesGenerated = morse_audio(elementsPtr, elementCount, audioBufferPtr, maxSamples, audioParamsPtr);

  // Copy audio data to JavaScript array
  const audioData = new Float32Array(samplesGenerated);
  for (let i = 0; i < samplesGenerated; i++) {
    audioData[i] = module.HEAPF32[(audioBufferPtr >> 2) + i];
  }

  // Cleanup
  module._free(timingParamsPtr);
  module._free(elementsPtr);
  module._free(audioBufferPtr);
  module._free(audioParamsPtr);

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