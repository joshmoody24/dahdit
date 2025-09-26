# dahdit/core

Core C implementation of the Morse code generator.

## Building

Requires a C compiler and make:

```bash
make
```

This builds the native binary. For WebAssembly builds used by the JavaScript bindings, see `../bindings/javascript/`.

## API

The core provides two main functions:

- `morse_timing()` - Converts text to morse timing elements
- `morse_audio()` - Generates audio samples from timing elements

See `morse.h` for full API documentation.