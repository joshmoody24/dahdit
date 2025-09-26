# dahdit/bindings

Language bindings for the dahdit Morse code generator.

## Available Languages

- **JavaScript** (`javascript/`) - Browser-ready ES modules with WebAssembly

## Adding New Languages

New language bindings should:

1. Compile the core C code to the target platform
2. Provide idiomatic API wrappers
3. Handle memory management appropriately
4. Include proper documentation and examples