# dahdit

Fast WebAssembly-based Morse code generator with prosign support.

## Features

- **Generation**: Complete ITU Morse code support (A-Z, 0-9, punctuation)
- **Interpretation**: Real-time morse code interpretation using K-means clustering
- **Interactive Interface**: Tap-based morse input with automatic translation
- **Prosign Support**: Using bracket syntax: `[SOS]`, `[AR]`, `[SK]`
- **High Quality Audio**: Clean waveforms with smooth attack/release envelopes

## Demo

Try it online: [https://joshmoody24.github.io/dahdit](https://joshmoody24.github.io/dahdit)

### Interactive Features

- **Morse Generator** (`index.html`): Convert text to morse code audio with customizable timing and audio parameters
- **Tap Morse** (`tap-morse.html`): Interactive morse code input interface - tap and hold to create dots and dashes, automatic interpretation after 3 seconds of silence

## Languages

### JavaScript

```bash
npm install dahdit
```

```javascript
import { generateMorseAudio, playMorseAudio, interpretMorseSignals, ready } from 'dahdit';

// Wait for WebAssembly to load (recommended)
await ready;

// Generate morse code audio
const audio = generateMorseAudio({ text: "HELLO WORLD [SOS]" });
playMorseAudio(audio);

// Interpret morse code signals
const signals = [
  { on: true, seconds: 0.18 },   // dash
  { on: false, seconds: 0.06 },  // gap
  { on: true, seconds: 0.06 },   // dot
  { on: false, seconds: 0.18 }   // end
];
const result = interpretMorseSignals({ signals });
console.log(result.text); // "N"
```

## Development

```bash
make test         # Run all tests (C core + JS bindings)
make build        # Build everything (core binary + WASM)
make dev          # Run tests then build everything
make clean        # Clean all build artifacts
```

For component-specific development, use the Makefiles in `core/` and `bindings/javascript/` directories.

## Project Structure

- `core/` - C implementation with WebAssembly build
- `bindings/javascript/` - JavaScript wrapper and npm package
- `bindings/` - Language bindings (currently JavaScript only)

## License

MIT