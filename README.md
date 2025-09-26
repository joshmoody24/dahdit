# dahdit

Fast WebAssembly-based Morse code generator with prosign support.

## Features

- Complete ITU Morse code support (A-Z, 0-9, punctuation)
- Prosign support using bracket syntax: `[SOS]`, `[AR]`, `[SK]`
- Clean audio with smooth attack/release envelopes

## Demo

Try it online: [https://joshmoody24.github.io/dahdit](https://joshmoody24.github.io/dahdit)

## Languages

### JavaScript

```bash
npm install dahdit
```

```javascript
import { generateMorseAudio, playMorseAudio, ready } from 'dahdit';

// Wait for WebAssembly to load (recommended)
await ready;

const audio = generateMorseAudio({ text: "HELLO WORLD [SOS]" });
playMorseAudio(audio);
```

## Project Structure

- `core/` - C implementation with WebAssembly build
- `bindings/javascript/` - JavaScript wrapper and npm package
- `bindings/` - Language bindings (currently JavaScript only)

## License

MIT