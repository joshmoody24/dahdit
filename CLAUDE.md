# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

This is a **multi-layer architecture** with a C core compiled to both native binaries and WebAssembly:

1. **Core C Implementation** (`core/`)
   - `morse.c` - Main morse code logic with O(1) character lookup table
   - `wav.c` - WAV file generation utilities
   - `morse.h` - Core API definitions and parameter structs
   - `main.c` - Native stress testing binary (not used in WebAssembly)

2. **WebAssembly Layer** (`bindings/javascript/`)
   - Only `morse.c` gets compiled to WASM (not `main.c`)
   - Emscripten generates `morse-wasm.js` and `morse-wasm.wasm`
   - ES6 modules with specific exported functions and memory heaps

3. **JavaScript API Layer** (`bindings/javascript/morse.js`)
   - Thin wrapper around WebAssembly with memory management
   - Converts between JS objects and C structs
   - Tree-shakable ES module exports: `generateMorseAudio`, `playMorseAudio`, `generateMorseTiming`, `ready`

## Core Functions & Data Flow

**Two-phase processing model:**
1. `morse_timing()` - Text → Timing elements (dots/dashes/gaps with durations)
2. `morse_audio()` - Timing elements → Audio samples with envelope processing

**Key C structs** (in `morse.h`):
- `MorseTimingParams` - Input parameters (WPM)
- `MorseAudioParams` - Audio parameters (sample rate, frequency, volume)
- `MorseElement` - Individual timing element (type + duration)

## Build Commands

**Native C development:**
```bash
cd core/
make              # Build native binary
make run          # Build and run stress tests
make clean        # Clean build artifacts
```

**WebAssembly development:**
```bash
cd bindings/javascript/
make              # Build WebAssembly modules
make clean        # Clean WASM artifacts
npm run build     # Same as make (for package.json compatibility)
```

**Development environment:**
```bash
nix develop       # Enter dev shell with gcc, make, emscripten
```

## Adding New Parameters

When adding parameters to core functions, **propagation is required across all layers**:

1. **Update C structs** in `morse.h` (e.g., `MorseTimingParams`, `MorseAudioParams`)
2. **Update default definitions** in `morse.h` (e.g., `MORSE_DEFAULT_TIMING_PARAMS`)
3. **Update JavaScript wrapper** in `bindings/javascript/morse.js`:
   - Memory allocation sizes for new struct members
   - Parameter destructuring in function signatures
   - HEAP memory writes to pass data to C
4. **Update JSDoc** parameter documentation in wrapper functions
5. **Update any language binding** in `bindings/*/` directories

Example: Adding `tone_shape` parameter to `MorseAudioParams` requires updating struct definition, default macro, JS memory allocation (from 12 to 16 bytes), and JS function parameter destructuring.

## Prosign Syntax

Text processing supports **bracket prosigns**: `[SOS]` generates `...---...` with 1-dot spacing between characters (not 3-dot). Spaces inside brackets are ignored. Regular text uses standard ITU spacing rules.

## Performance Characteristics

- **Character lookup**: O(1) via direct array indexing `morse_patterns[ch]`
- Supports both uppercase and lowercase via duplicate table entries
- **Timing generation**: CPU-bound, scales linearly
- **Audio generation**: Memory/math-bound due to floating-point calculations

## Deployment

**GitHub Pages**: Automatic deployment via `.github/workflows/pages.yml` builds WASM and deploys to `https://joshmoody24.github.io/dahdit`

**npm publishing**: From `bindings/javascript/` directory, `npm publish` after `npm run build`