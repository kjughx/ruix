use crate::__trace;
use core::arch::asm;

#[no_mangle]
#[naked]
#[link_section = ".start"]
extern "C" fn _start() -> ! {
    unsafe {
        asm!(
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
            "mov al, 00010001b",
            "out 0x20, al",
            "mov al, 0x20",
            "out 0x21, al",
            "mov al, 00000001b",
            "out 0x21, al",
            "call kmain",
            "42:",
            "hlt",
            "jmp 42b",
            options(noreturn)
        );
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        __trace!("[{}:{}] panic - {}", loc.file(), loc.line(), info.message());
    } else {
        __trace!("Kernel panic somwhere!");
    }

    unsafe { asm!("hlt", options(noreturn)) }
}
