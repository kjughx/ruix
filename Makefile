SRC=src
BIN=bin
OBJ=build

BINS=$(BIN)/kernel.bin

all: dev

image:
	@rm -f $(BIN)/os.bin
	@dd status=none if=$(BIN)/kernel.bin >> $(BIN)/os.bin
	@dd status=none if=/dev/zero bs=1024 count=1024 >> $(BIN)/os.bin
	@sudo mount $(BIN)/os.bin /mnt/d
	@sudo cp shell/shell /mnt/d/SHELL
	@sudo umount /mnt/d

dev: prelude kdev std_dev image

kdev:
	@cargo build -p kernel
	@cp $(OBJ)/i686-unknown-none/debug/kernel build/kernelfull.o
	@objcopy --target elf32-i386 -O binary build/kernelfull.o $(BIN)/kernel.bin

std_dev:
	@cargo build -p std
	@cp $(OBJ)/i686-unknown-none/debug/libstd.a build/libstd.a

release: prelude krelease std_rel image

krelease:
	@cargo build --release -p kernel
	@cp $(OBJ)/i686-unknown-none/release/kernel build/kernelfull.o
	@objcopy --target elf32-i386 -O binary build/kernelfull.o $(BIN)/kernel.bin

std_rel:
	@cargo build --release -p std
	@cp $(OBJ)/i686-unknown-none/release/libstd.a build/libstd.a

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
		-ex "set output-radix 16" \
		-ex="target remote | qemu-system-i386 -display none -S -gdb stdio -hda bin/os.bin" \
		-ex="add-symbol-file build/kernelfull.o 0x101000" \
		-ex="break kmain"

.PHONY: qemu
qemu: dev
	qemu-system-i386 -hda bin/os.bin -serial stdio
	# qemu-system-i386 -hda bin/os.bin -monitor stdio

.PHONY: trace
trace: dev
	qemu-system-i386 -hda bin/os.bin -serial stdio -display none

.PHONY: gdb
debug: dev
	sh scripts/debug.sh
