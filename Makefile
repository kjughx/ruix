SRC=src
BIN=bin
OBJ=build

TARGET=i686-elf
CFLAGS=-std=gnu99 -Wall -Werror -Wno-cpp -O0 -Isrc/ -Iinc/ -ffreestanding -falign-jumps -falign-functions -falign-labels -falign-loops -fstrength-reduce -fomit-frame-pointer -finline-functions -fno-builtin -nostartfiles -nodefaultlibs -nostdlib
LDFLAGS=-g -relocatable
CC=gcc

BOOT_SRCS = boot/boot.asm
BINS=$(BIN)/boot.bin $(BIN)/kernel.bin

R_SRCS := $(shell find src -name "*.rs")
C_SRCS := $(shell find src -name "*.c")
ASM_SRCS := $(filter-out src/boot/boot.asm, $(shell find src -name "*.asm"))

OBJS = $(patsubst src/%.asm, $(OBJ)/%.asm.o, $(ASM_SRCS))

all: prelude $(BINS)
	@rm -f $(BIN)/os.bin
	@dd status=none if=$(BIN)/boot.bin >> $(BIN)/os.bin
	@dd status=none if=$(BIN)/kernel.bin >> $(BIN)/os.bin
	@dd status=none if=/dev/zero bs=1024 count=1024 >> $(BIN)/os.bin

$(BIN)/boot.bin: $(SRC)/boot/boot.asm
	nasm -f bin $< -o $@

$(OBJ)/%.asm.o: $(SRC)/%.asm
	@mkdir -p $(dir $@)
	nasm -f elf $< -o $@

$(BIN)/kernel.bin: rust

.PHONY: rust
rust: $(OBJS) $(R_SRCS) $(C_SRCS)
	@cargo build
	@cp $(OBJ)/i686-unknown-none/debug/ruix build/kernelfull.o
	@objcopy --target elf32-i386 -O binary build/kernelfull.o $(BIN)/kernel.bin

.PHONY: prelude
prelude:
	@mkdir -p build
	@mkdir -p bin

.PHONY: clean
clean:
	@rm -rf $(OBJ)/* $(BIN)/*

.PHONY: gdb
gdb: all
	gdb --command=debug.gdb

.PHONY: qemu
qemu: all
	qemu-system-i386 -hda bin/os.bin -serial stdio

.PHONY: trace
trace: all
	qemu-system-i386 -hda bin/os.bin -serial stdio -display none
