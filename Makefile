SRC=src
BIN=bin
OBJ=build

BOOT_SRCS = boot/boot.asm
BINS=$(BIN)/boot.bin $(BIN)/kernel.bin

SRCS := $(shell find src -name "*.rs")

all: dev

image: $(BINS)
	@rm -f $(BIN)/os.bin
	@dd status=none if=$(BIN)/boot.bin >> $(BIN)/os.bin
	@dd status=none if=$(BIN)/kernel.bin >> $(BIN)/os.bin
	@dd status=none if=/dev/zero bs=1024 count=1024 >> $(BIN)/os.bin

dev: prelude _dev image
_dev: $(SRCS)
	@cargo build
	@cp $(OBJ)/i686-unknown-none/debug/kernel build/kernelfull.o
	@objcopy --target elf32-i386 -O binary build/kernelfull.o $(BIN)/kernel.bin

release: prelude _release image
_release: $(SRCS)
	@cargo build --release
	@cp $(OBJ)/i686-unknown-none/release/kernel build/kernelfull.o
	@objcopy --target elf32-i386 -O binary build/kernelfull.o $(BIN)/kernel.bin


$(BIN)/boot.bin: $(SRC)/boot/boot.asm
	nasm -f bin $< -o $@

.PHONY: prelude
prelude:
	@mkdir -p build
	@mkdir -p bin

.PHONY: clean
clean:
	@rm -rf $(OBJ)/* $(BIN)/*

.PHONY: gdb
gdb: dev
	rust-gdb \
		-ex "set confirm off" \
		-ex="target remote | qemu-system-i386 -display none -S -gdb stdio -hda bin/os.bin" \
		-ex="add-symbol-file build/kernelfull.o 0x101000" \
		-ex="break kmain"


.PHONY: qemu
qemu: dev
	qemu-system-i386 -hda bin/os.bin -serial stdio

.PHONY: trace
trace: dev
	qemu-system-i386 -hda bin/os.bin -serial stdio -display none
