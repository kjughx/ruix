#![no_std]
#![no_main]

use ruix::{
    disk::Disk,
    fs::Vfs,
    gdt::GDT,
    idt::IDT,
    paging::{KernelPage, Paging},
    println, traceln,
    tty::Terminal,
};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    Terminal::init();

    GDT::load();

    IDT::load();

    // Resolve the connected disks
    match Vfs::resolve(Disk::get_mut(0)) {
        Ok(()) => (),
        Err(_) => println!("Could not resolve disk 0"),
    }

    KernelPage::switch();
    Paging::enable();

    println!("Hello, World!");
    traceln!("Hello, World!");
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}
