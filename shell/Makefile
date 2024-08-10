CFLAGS := -std=gnu99 -Wall -Wextra -Werror -Wno-cpp -Wno-unused-parameter -Wno-unused-function -O0 \
	-I../crates/std/ -ffreestanding \
	-falign-jumps -falign-functions -falign-labels -falign-loops -fstrength-reduce  -fomit-frame-pointer -finline-functions \
	-fno-zero-initialized-in-bss -fno-builtin -nostartfiles -nodefaultlibs -nostdlib -nostdinc -Wl,--gc-sections -Wl,--build-id=none

# LDFLAGS := -g
CC := gcc -m32
LD := ld -b elf32-t--oformat elf32-i386

C_SRCS := $(shell find  -name "*.c")

OBJS += $(patsubst %.c, %.o, $(C_SRCS))

all: $(OBJS)
	$(CC) $(CFLAGS) -Tlinker.ld -o shell -static -O0 -nostdlib -fpic $< ../build/libstd.a
	@strip shell

%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	@rm -rf *.o shell
