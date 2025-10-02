CC = gcc
CFLAGS = -Wall -O2
OBJS = main.o morse.o wav.o interpret_morse.o
TEST_OBJS = test.o morse.o wav.o interpret_morse.o

all: morse

morse: $(OBJS)
	$(CC) $(CFLAGS) -o $@ $(OBJS) -lm

test: test-runner
	./test-runner

test-runner: $(TEST_OBJS)
	$(CC) $(CFLAGS) -o $@ $(TEST_OBJS) -lm

run: morse
	./morse

clean:
	rm -rf *.o morse test-runner

.PHONY: all test run clean
