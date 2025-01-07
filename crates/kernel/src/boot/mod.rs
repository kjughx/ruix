core::arch::global_asm!(include_str!("x86.S"), options(att_syntax));

#[no_mangle]
#[naked]
#[link_section = ".start"]
extern "C" fn _start() -> ! {
    unsafe {
        core::arch::naked_asm!(
            ".code32",
            "mov ax, 0x10", // Data segment is at offset 0x10
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            "mov ebp, 0x00200000",
            "mov esp, ebp",
            "in al, 0x92",
            "or al, 2",
            "out 0x92, al",
            "mov al, 0b00010001",
            "out 0x20, al",
            "mov al, 0x20",
            "out 0x21, al",
            "mov al, 0b00000001",
            "out 0x21, al",
            "call kmain",
            "42:",
            "hlt",
            "jmp 42b",
        );
    }
}
