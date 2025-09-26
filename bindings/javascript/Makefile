CC = emcc
CFLAGS = -O3 -s WASM=1 -s EXPORTED_FUNCTIONS='["_morse_timing", "_morse_audio", "_malloc", "_free"]' \
         -s EXPORTED_RUNTIME_METHODS='["ccall", "cwrap", "HEAPU8", "HEAPF32", "HEAP32"]' \
         -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=1 -s EXPORT_NAME="MorseWasmModule" \
         -s EXPORT_ES6=1

CORE_DIR = ../../core
SOURCES = $(CORE_DIR)/morse.c

all: morse.js

morse-wasm.js: $(SOURCES)
	$(CC) $(CFLAGS) $(SOURCES) -o morse-wasm.js

morse.js: morse-wasm.js

clean:
	rm -f morse-wasm.js morse-wasm.wasm

.PHONY: all clean