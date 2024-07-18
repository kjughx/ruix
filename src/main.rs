#![no_std]
#![no_main]

use ruix::{
    disk::Disk,
    fs::Vfs,
    gdt::GDT,
    idt::IDT,
    paging::{KernelPage, Paging},
    traceln,
    tty::{init_screen, print},
};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    init_screen();

    GDT::load();

    IDT::load();

    // Resolve the connected disks
    match Vfs::resolve(Disk::get_mut(0)) {
        Ok(()) => (),
        Err(_) => print("Could not resolve disk 0"),
    }

    Paging::switch(KernelPage::get());
    Paging::enable();

    print("Hello, World!");
    traceln!("Hello, World!");
    loop {
        unsafe { core::arch::asm!("hlt", options(noreturn)) };
    }
}
