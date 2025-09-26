# dahdit

Fast WebAssembly-based Morse code generator for browsers.

## Installation

```bash
npm install dahdit
```

## Usage

```javascript
import { generateMorseAudio, playMorseAudio, ready } from 'dahdit';

// Wait for WebAssembly to load (recommended)
await ready;

// Generate audio data
const audio = generateMorseAudio({
  text: "HELLO WORLD [SOS]",
  wpm: 20,
  frequency: 440,
  volume: 0.5
});

// Play in browser
playMorseAudio(audio);
```

## API

### `generateMorseAudio(params)`

Generates Morse code audio data.

**Parameters:**
- `text` (string) - Text to convert to Morse code
- `wpm` (number, optional) - Words per minute (default: 20)
- `sampleRate` (number, optional) - Audio sample rate (default: 22050)
- `frequency` (number, optional) - Tone frequency in Hz (default: 440)
- `volume` (number, optional) - Volume 0.0-1.0 (default: 0.5)

**Returns:** Audio data object with `audioData`, `sampleRate`, and `duration` properties.

### `playMorseAudio(audioResult)`

Plays audio data using Web Audio API.

**Parameters:**
- `audioResult` - Object returned from `generateMorseAudio()`

**Returns:** AudioBufferSourceNode for controlling playback.

### `ready`

Promise that resolves when WebAssembly module is loaded.

## Prosigns

Use `[brackets]` for prosigns like `[SOS]`, `[AR]`, `[SK]`. These run letters together without normal inter-character spacing.

Learn more: [Prosigns for Morse code](https://en.wikipedia.org/wiki/Prosigns_for_Morse_code)