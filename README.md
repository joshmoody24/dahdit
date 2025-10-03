# dahdit

Fast WebAssembly-based Morse code generator with prosign support.

## Features

- **Generation**: Complete ITU Morse code support (A-Z, 0-9, punctuation)
- **Interpretation**: Morse code interpretation (coming soon)
- **Interactive Interface**: Tap-based morse input with automatic translation
- **Prosign Support**: Using bracket syntax: `[SOS]`, `[AR]`, `[SK]`
- **High Quality Audio**: Clean waveforms with smooth attack/release envelopes

## Demo

Try it online: [https://joshmoody24.github.io/dahdit](https://joshmoody24.github.io/dahdit)

### Interactive Features

- **Morse Generator** (`index.html`): Convert text to morse code audio with customizable timing and audio parameters
- **Tap Morse** (`tap-morse.html`): Interactive morse code input interface - tap and hold to create dots and dashes, automatic interpretation after 3 seconds of silence

## Usage

### Rust (Core Library)

```rust
use dahdit::{morse_timing, morse_audio, MorseTimingParams, MorseAudioParams};

let params = MorseTimingParams {
    wpm: 20,
    word_gap_multiplier: 1.0,
    humanization_factor: 0.0,
    random_seed: 0,
};

let elements = morse_timing("HELLO WORLD [SOS]", &params)?;
let (audio_data, duration) = morse_audio(&elements, &audio_params)?;
```

### JavaScript (via WebAssembly)

```bash
npm install dahdit
```

```javascript
import {
  generateMorseAudio,
  playMorseAudio,
  MorseAudioMode,
  MorseWaveformType,
} from "dahdit";

// Generate and play morse code audio
const audio = generateMorseAudio({
  text: "HELLO WORLD [SOS]",
  audioMode: MorseAudioMode.Radio,
  waveformType: MorseWaveformType.Sine,
});
playMorseAudio(audio);
```

## Development

### Universal Commands (Root Level)

```bash
make test         # Run all tests (Rust + JavaScript)
make build        # Build everything
make dev          # Format, lint, test, then build
make format       # Format all code
make clean        # Clean all build artifacts
```

### Rust (Core)

```bash
cd core/
cargo test        # Run Rust tests
cargo fmt         # Format code
cargo clippy      # Lint code
```

### JavaScript (User Package)

```bash
cd bindings/javascript/wrapper/
npm test          # Run JavaScript tests
npm run build     # Build WASM from Rust
npm run format    # Format code
```

## Project Structure

- `core/` - Rust implementation with WebAssembly bindings
- `bindings/javascript/wasm-core/` - Generated WASM package (don't edit)
- `bindings/javascript/wrapper/` - User-facing JavaScript package

## License

MIT
