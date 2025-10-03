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
  generate_morse_timing,
  generate_morse_audio,
  interpret_morse_signals,
  MorseAudioMode,
  MorseWaveformType,
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
 * Morse audio modes for different sound characteristics
 * @readonly
 * @enum {number}
 */
export const AudioMode = MorseAudioMode;

/**
 * Waveform types for radio mode audio generation
 * @readonly
 * @enum {number}
 */
export const WaveformType = MorseWaveformType;

/**
 * @typedef {Object} MorseTimingElement
 * @property {string} type - Element type: "dot", "dash", or "gap"
 * @property {number} duration_seconds - Duration in seconds
 */

/**
 * @typedef {Object} TimingConfig
 * @property {number} [wpm=20] - Words per minute (5-50 recommended)
 * @property {number} [wordGapMultiplier=1.0] - Multiplier for word gap duration
 * @property {number} [humanizationFactor=0.0] - Amount of timing variation (0.0-1.0)
 * @property {number} [randomSeed=0] - Random seed for consistent humanization
 */

/**
 * Generate morse code timing elements from text
 *
 * @param {string} text - The text to convert to morse code
 * @param {TimingConfig} [config={}] - Optional timing configuration
 * @returns {Array<MorseTimingElement>} Array of timing elements with type and duration
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
  if (
    config.wpm !== undefined &&
    (!Number.isInteger(config.wpm) || config.wpm <= 0 || config.wpm > 100)
  ) {
    throw new Error("WPM must be a positive integer between 1-100");
  }

  // Convert config object to JSON string for Rust
  const configJson = JSON.stringify(config);
  const result = generate_morse_timing(text, configJson);
  const elements = result.elements;
  result.free(); // Clean up Rust memory
  return elements;
}

/**
 * @typedef {Object} BaseAudioConfig
 * @property {number} [wpm=20] - Words per minute for timing
 * @property {number} [wordGapMultiplier=1.0] - Multiplier for word gap duration
 * @property {number} [humanizationFactor=0.0] - Amount of timing variation (0.0-1.0)
 * @property {number} [randomSeed=0] - Random seed for consistent variation
 * @property {number} [sampleRate=44100] - Audio sample rate in Hz
 * @property {number} [volume=0.5] - Audio volume (0.0-1.0)
 * @property {number} [lowPassCutoff=20000] - Low-pass filter frequency in Hz
 * @property {number} [highPassCutoff=20] - High-pass filter frequency in Hz
 */

/**
 * @typedef {BaseAudioConfig & {
 *   audioMode: typeof AudioMode.Radio,
 *   freqHz?: number,
 *   waveformType?: WaveformType,
 *   backgroundStaticLevel?: number
 * }} RadioAudioConfig
 * @property {number} [freqHz=440] - Radio frequency in Hz
 * @property {WaveformType} [waveformType=WaveformType.Sine] - Waveform type for radio tone
 * @property {number} [backgroundStaticLevel=0.0] - Background static level (0.0-1.0)
 */

/**
 * @typedef {BaseAudioConfig & {
 *   audioMode: typeof AudioMode.Telegraph,
 *   clickSharpness?: number,
 *   resonanceFreq?: number,
 *   decayRate?: number,
 *   mechanicalNoise?: number,
 *   solenoidResponse?: number,
 *   roomToneLevel?: number,
 *   reverbAmount?: number
 * }} TelegraphAudioConfig
 * @property {number} [clickSharpness=0.5] - Click sharpness factor (0.0-1.0)
 * @property {number} [resonanceFreq=800] - Resonance frequency in Hz
 * @property {number} [decayRate=10] - Decay rate for telegraph clicks
 * @property {number} [mechanicalNoise=0.1] - Mechanical noise level (0.0-1.0)
 * @property {number} [solenoidResponse=0.7] - Solenoid response factor
 * @property {number} [roomToneLevel=0.05] - Room tone level (0.0-1.0)
 * @property {number} [reverbAmount=0.3] - Reverb amount (0.0-1.0)
 */

/**
 * @typedef {Object} MorseAudioResult
 * @property {Float32Array} audioData - Raw audio sample data
 * @property {number} sampleRate - Sample rate in Hz
 * @property {number} duration - Duration in seconds
 */

/**
 * Generate morse code audio from text
 *
 * @param {string} text - The text to convert to morse code audio
 * @param {BaseAudioConfig | RadioAudioConfig | TelegraphAudioConfig} [config={}] - Audio configuration
 * @returns {MorseAudioResult} Object with audio data, sample rate, and duration
 * @throws {Error} If text is invalid or parameters are out of range
 *
 * @example
 * // Basic usage with default radio mode
 * const audio = generateMorseAudio("HELLO WORLD");
 *
 * @example
 * // Radio mode with custom frequency and waveform
 * const audio = generateMorseAudio("CQ CQ", {
 *   audioMode: AudioMode.Radio,
 *   freqHz: 600,
 *   waveformType: WaveformType.Square,
 *   backgroundStaticLevel: 0.1
 * });
 *
 * @example
 * // Telegraph mode with mechanical characteristics
 * const audio = generateMorseAudio("SOS", {
 *   audioMode: AudioMode.Telegraph,
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

  // Convert config object to JSON string for Rust
  const configJson = JSON.stringify(config);
  const result = generate_morse_audio(text, configJson);
  const audioData = result.audio_data;
  const sampleRate = result.sample_rate;
  const duration = result.duration;
  result.free(); // Clean up Rust memory
  return { audioData, sampleRate, duration };
}

/**
 * @typedef {Object} AudioPlayer
 * @property {() => void} stop - Stop audio playback
 * @property {boolean} playing - True if audio is currently playing
 */

/**
 * Play morse code audio in the browser using Web Audio API
 *
 * @param {MorseAudioResult} audioResult - Audio data from generateMorseAudio()
 * @param {Object} [config={}] - Optional playback configuration
 * @returns {AudioPlayer} Playback controller with stop() method and playing getter
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
 * @typedef {Object} MorseSignal
 * @property {boolean} on - True for tone on (dot/dash), false for gap
 * @property {number} seconds - Duration of signal in seconds
 */

/**
 * @typedef {Object} InterpretConfig
 * @property {number} [maxOutputLength=1000] - Maximum output text length
 */

/**
 * @typedef {Object} MorseInterpretResult
 * @property {string} text - Decoded text from morse signals
 * @property {number} confidence - Confidence score (0.0-1.0)
 * @property {number} signalsProcessed - Number of signals processed
 * @property {number} patternsRecognized - Number of patterns recognized
 */

/**
 * Interpret morse code signals and convert them back to text
 *
 * @param {Array<MorseSignal>} signals - Array of morse signal objects
 * @param {InterpretConfig} [config={}] - Optional interpretation parameters
 * @returns {MorseInterpretResult} Interpretation result with text, confidence, and statistics
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

  // Convert config object to JSON string for Rust
  const configJson = JSON.stringify(config);
  const signalsJson = JSON.stringify(signals);

  const result = interpret_morse_signals(signalsJson, configJson);

  // Create JavaScript object from Rust result
  const jsResult = {
    text: result.text,
    confidence: result.confidence,
    signalsProcessed: result.signals_processed,
    patternsRecognized: result.patterns_recognized,
  };

  result.free(); // Clean up Rust memory
  return jsResult;
}
