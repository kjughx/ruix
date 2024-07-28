target remote | qemu-system-i386 -display none -S -gdb stdio -hda bin/os.bin
add-symbol-file build/kernelfull.o 0x101000

