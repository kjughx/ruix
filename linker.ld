ENTRY(_start)
SECTIONS
{
    . = 1M;
    .start : ALIGN(4096) {
        *(.start)
        . = 1M + 512;
    } =0

    .text : ALIGN(4096) {
        *(.text)
        *(.text.*)
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
