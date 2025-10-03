// Universal Rust-based WebAssembly Morse code library
// Works in both Node.js 18+ and browsers

// Environment detection for imports
const isNode = typeof process !== "undefined" && process.versions?.node;

// Import WASM module based on environment
let wasmModule;
if (isNode) {
  wasmModule = await import("morse-core");
} else {
  wasmModule = await import("../wasm-core/morse_core.js");
}

// Extract what we need from WASM module
const {
  default: wasmInit,
  generate_morse_timing,
  generate_morse_audio,
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
  const wasmPath = path.join(__dirname, "../wasm-core/morse_core_bg.wasm");
  const wasmBytes = fs.readFileSync(wasmPath);
  await wasmInit(wasmBytes);
} else {
  // Browser environment - let wasm-pack handle the fetch
  await wasmInit();
}

// Export Rust enums
export { MorseAudioMode, MorseWaveformType };

// Export with our preferred names and add memory cleanup
export function generateMorseTiming(config) {
  // Basic validation
  if (!config.text || typeof config.text !== "string") {
    throw new Error("Invalid text input");
  }
  if (
    config.wpm !== undefined &&
    (!Number.isInteger(config.wpm) || config.wpm <= 0)
  ) {
    throw new Error("WPM must be a positive integer");
  }

  // Convert config object to JSON string for Rust
  const configJson = JSON.stringify(config);
  const result = generate_morse_timing(config.text, configJson);
  const elements = result.elements;
  result.free(); // Clean up Rust memory
  return elements;
}

export function generateMorseAudio(config) {
  // Basic validation
  if (!config.text || typeof config.text !== "string") {
    throw new Error("Invalid text input");
  }

  // Convert config object to JSON string for Rust
  const configJson = JSON.stringify(config);
  const result = generate_morse_audio(config.text, configJson);
  const audioData = result.audio_data;
  const sampleRate = result.sample_rate;
  const duration = result.duration;
  result.free(); // Clean up Rust memory
  return { audioData, sampleRate, duration };
}

// Browser audio playback using Web Audio API
export function playMorseAudio(audioResult) {
  if (typeof AudioContext === "undefined") {
    throw new Error(
      "playMorseAudio not available in Node.js environment - use in browser instead",
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

  // Return object compatible with existing browser code
  return {
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
    get playing() {
      return isPlaying;
    },
  };
}

// Placeholder for interpretation
export function interpretMorseSignals(params) {
  throw new Error("Morse interpretation not implemented yet in Rust version");
}
