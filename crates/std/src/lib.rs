#![no_std]
#![no_main]
#![feature(naked_functions)]

extern crate syscalls;

extern "C" {
    pub fn main() -> usize;
}

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    unsafe {
        core::arch::asm!("call main", "push eax", "call {}", sym syscalls::exit, options(noreturn))
    };
}

#[inline(never)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { syscalls::exit(1) };
    unreachable!()
}
