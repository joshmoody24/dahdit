CC = gcc
CFLAGS = -Wall -O2
OBJS = main.o morse.o wav.o

all: morse

morse: $(OBJS)
	$(CC) $(CFLAGS) -o $@ $(OBJS) -lm

run: morse
	./morse

clean:
	rm -rf *.o morse
