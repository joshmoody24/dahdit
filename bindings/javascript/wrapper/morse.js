// Universal Rust-based WebAssembly Morse code library
// Works in both Node.js 18+ and browsers

// Environment detection for imports
const isNode = typeof process !== "undefined" && process.versions?.node;

// Import WASM module based on environment
let wasmModule;
if (isNode) {
  wasmModule = await import("morse-wasm");
} else {
  wasmModule = await import("../wasm-core/morse_wasm.js");
}

// Extract what we need from WASM module
const {
  default: wasmInit,
  morse_timing_json,
  morse_audio_json,
  morse_interpret_json,
} = wasmModule;

// Initialize WASM immediately
if (isNode) {
  // Node.js environment - load WASM directly
  const fs = await import("fs");
  const path = await import("path");
  const { fileURLToPath } = await import("url");
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const wasmPath = path.join(__dirname, "../wasm-core/morse_wasm_bg.wasm");
  const wasmBytes = fs.readFileSync(wasmPath);
  await wasmInit(wasmBytes);
} else {
  // Browser environment - let wasm-pack handle the fetch
  await wasmInit();
}

/**
 * Generate morse code timing elements from text
 *
 * @param {string} text - The text to convert to morse code
 * @param {Object} [config={}] - Optional timing configuration
 * @returns {Array<Object>} Array of timing elements with type and duration
 * @throws {Error} If text is invalid or parameters are out of range
 *
 * @example
 * // Basic usage
 * const elements = generateMorseTiming("HELLO");
 *
 * @example
 * // With custom speed and humanization
 * const elements = generateMorseTiming("SOS", {
 *   wpm: 15,
 *   humanizationFactor: 0.1
 * });
 */
export function generateMorseTiming(text, config = {}) {
  // Basic validation
  if (!text || typeof text !== "string") {
    throw new Error("Text must be a non-empty string");
  }

  const configJson = JSON.stringify(config);
  const resultJson = morse_timing_json(text, configJson);
  return JSON.parse(resultJson);
}

/**
 * Generate morse code audio from text
 *
 * @param {string} text - The text to convert to morse code audio
 * @param {Object} [config={}] - Audio configuration
 * @returns {Object} Object with audioData, sampleRate, duration, and elements
 * @throws {Error} If text is invalid or parameters are out of range
 *
 * @example
 * // Basic usage with default radio mode
 * const audio = generateMorseAudio("HELLO WORLD");
 *
 * @example
 * // Radio mode with custom frequency and waveform
 * const audio = generateMorseAudio("CQ CQ", {
 *   audioMode: "radio",
 *   freqHz: 600,
 *   waveformType: "square",
 *   backgroundStaticLevel: 0.1
 * });
 *
 * @example
 * // Telegraph mode with mechanical characteristics
 * const audio = generateMorseAudio("SOS", {
 *   audioMode: "telegraph",
 *   wpm: 12,
 *   clickSharpness: 0.8,
 *   mechanicalNoise: 0.15,
 *   reverbAmount: 0.4
 * });
 */
export function generateMorseAudio(text, config = {}) {
  // Basic validation
  if (!text || typeof text !== "string") {
    throw new Error("Text must be a non-empty string");
  }

  const configJson = JSON.stringify(config);
  const resultJson = morse_audio_json(text, configJson);
  const result = JSON.parse(resultJson);

  return {
    audioData: new Float32Array(result.audioData),
    sampleRate: result.sampleRate,
    duration: result.duration,
    elements: result.elements,
  };
}

/**
 * Play morse code audio in the browser using Web Audio API
 *
 * @param {Object} audioResult - Audio data from generateMorseAudio()
 * @param {Object} [config={}] - Optional playback configuration
 * @returns {Object} Playback controller with stop() method and playing getter
 * @throws {Error} If not in browser environment or audio data is invalid
 *
 * @example
 * // Generate and play morse audio
 * const audio = generateMorseAudio("HELLO");
 * const player = playMorseAudio(audio);
 *
 * // Stop playback early if needed
 * setTimeout(() => player.stop(), 2000);
 */
export function playMorseAudio(audioResult, config = {}) {
  if (typeof AudioContext === "undefined") {
    throw new Error(
      "playMorseAudio not available in Node.js environment - use in browser instead",
    );
  }

  if (!audioResult || !audioResult.audioData || !audioResult.sampleRate) {
    throw new Error(
      "Invalid audio result - must have audioData and sampleRate",
    );
  }

  // Create audio context
  const audioContext = new (window.AudioContext || window.webkitAudioContext)();
  let source = null;
  let isPlaying = false;

  // Resume context if suspended (required after user interaction)
  if (audioContext.state === "suspended") {
    audioContext.resume();
  }

  // Create audio buffer
  const audioBuffer = audioContext.createBuffer(
    1, // mono
    audioResult.audioData.length,
    audioResult.sampleRate,
  );

  // Copy audio data to buffer
  const channelData = audioBuffer.getChannelData(0);
  channelData.set(audioResult.audioData);

  // Create audio source
  source = audioContext.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(audioContext.destination);

  // Set up ended callback
  source.onended = () => {
    isPlaying = false;
  };

  // Play audio
  source.start();
  isPlaying = true;

  // Return playback controller
  return {
    /**
     * Stop audio playback
     */
    stop() {
      if (source && isPlaying) {
        try {
          source.stop();
          isPlaying = false;
        } catch (e) {
          // Source might have already ended naturally
          isPlaying = false;
        }
      }
    },
    /**
     * Check if audio is currently playing
     * @returns {boolean} True if playing, false otherwise
     */
    get playing() {
      return isPlaying;
    },
  };
}

/**
 * Interpret morse code signals and convert them back to text
 *
 * @param {Array<Object>} signals - Array of morse signal objects with 'on' (boolean) and 'seconds' (number)
 * @param {Object} [config={}] - Optional interpretation parameters
 * @returns {Object} Interpretation result with text, confidence, and statistics
 * @throws {Error} If signals array is invalid
 *
 * @example
 * // Interpret simple morse signals
 * const signals = [
 *   { on: true, seconds: 0.1 },   // dot
 *   { on: false, seconds: 0.3 },  // gap
 *   { on: true, seconds: 0.3 },   // dash
 * ];
 * const result = interpretMorseSignals(signals);
 * console.log(result.text); // "ET" or similar
 *
 * @example
 * // With configuration
 * const result = interpretMorseSignals(signals, {
 *   maxOutputLength: 500
 * });
 */
export function interpretMorseSignals(signals, config = {}) {
  // Basic validation
  if (!signals || !Array.isArray(signals)) {
    throw new Error("Signals must be an array of signal objects");
  }

  // Validate signal objects
  for (let i = 0; i < signals.length; i++) {
    const signal = signals[i];
    if (typeof signal.on !== "boolean" || typeof signal.seconds !== "number") {
      throw new Error(
        `Invalid signal at index ${i}: must have 'on' (boolean) and 'seconds' (number) properties`,
      );
    }
    if (signal.seconds < 0) {
      throw new Error(
        `Invalid signal at index ${i}: 'seconds' must be non-negative`,
      );
    }
  }

  // Convert to JSON strings for Rust
  const configJson = JSON.stringify(config);
  const signalsJson = JSON.stringify(signals);

  const resultJson = morse_interpret_json(signalsJson, configJson);
  const result = JSON.parse(resultJson);

  // Convert snake_case to camelCase for JavaScript consistency
  return {
    text: result.text,
    confidence: result.confidence,
    signalsProcessed: result.signals_processed,
    patternsRecognized: result.patterns_recognized,
  };
}
