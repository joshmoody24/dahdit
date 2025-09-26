CC = emcc
CFLAGS = -O3 -s WASM=1 -s EXPORTED_FUNCTIONS='["_morse_new", "_morse_free", "_morse_set_i32", "_morse_set_f32", "_morse_set_str", "_morse_timing_size_ctx", "_morse_timing_fill_ctx", "_morse_audio_size_ctx", "_morse_audio_fill_ctx", "_malloc", "_free"]' \
         -s EXPORTED_RUNTIME_METHODS='["ccall", "cwrap", "HEAPU8", "HEAPF32", "HEAP32"]' \
         -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=1 -s EXPORT_NAME="MorseWasmModule" \
         -s EXPORT_ES6=1

CORE_DIR = ../../core
ABI_DIR = ../abi-wasm
SOURCES = $(CORE_DIR)/morse.c $(ABI_DIR)/morse_abi.c

all: morse.js

morse-wasm.js: $(SOURCES)
	$(CC) $(CFLAGS) $(SOURCES) -o morse-wasm.js

morse.js: morse-wasm.js

clean:
	rm -f morse-wasm.js morse-wasm.wasm

.PHONY: all clean