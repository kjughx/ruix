ENTRY(_entry)
SECTIONS
{
    . = 0x7c00;
    .boot : AT(0x0) ALIGN(512) {
        KEEP(*(.boot))
    }
    ASSERT(SIZEOF(.boot) <= 512, "Boot sector exceeds 512 bytes!")

    # This is loaded at 0x100000 but the binary position
    # should be 512 bytes after the boot-sector which ends at 0x1ff
    . = 1M;
    .start : AT(0x200) {
        KEEP(*(.start))
    } =0

    .text : ALIGN(4096) {
        *(.text)
        *(.text.*)
    }

    .isrs :  {
        PROVIDE(START_ISRS = .);

        KEEP(*(.isr.*))

        PROVIDE(END_ISRS = .);
    }

    .rodata : ALIGN(4096) {
        *(.rodata)
        *(.rodata..*)
    }

    .data : ALIGN(4096) {
        *(.data)
        *(.data.*)
    }

    .bss : ALIGN(4096) {
        *(COMMON)
        *(.bss)
    }
}
