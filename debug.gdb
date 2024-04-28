target remote | qemu-system-i386 -S -gdb stdio -hda bin/os.bin
add-symbol-file build/kernelfull.o 0x100200

