use core::arch::asm;

#[no_mangle]
pub static DATA_SEG: u32 = 0x10;

#[no_mangle]
#[naked]
#[link_section = ".start"]
extern "C" fn _start() -> ! {
    unsafe {
        asm!(
            ".code32",
            "mov ax, DATA_SEG",
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
            "mov al, 00010001b",
            "out 0x20, al",
            "mov al, 0x20",
            "out 0x21, al",
            "mov al, 00000001b",
            "out 0x21, al",
            "call kernel_main",
            "hlt",
            options(noreturn)
        );
    }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { asm!("hlt", options(noreturn)) }
}


