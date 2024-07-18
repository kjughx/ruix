#![no_std]
#![no_main]

use ruix::{
    disk::Disk,
    fs::Vfs,
    gdt::GDT,
    idt::IDT,
    paging::{KernelPage, Paging},
    println,
    process::Process,
    traceln,
    tty::Terminal,
};

#[no_mangle]
extern "C" fn kmain() -> ! {
    Terminal::init();

    GDT::load();

    IDT::load();

    // Resolve the connected disks
    let disk = Disk::get_mut(0);
    match Vfs::resolve(disk) {
        Ok(()) => (),
        Err(_) => println!("Could not resolve disk 0"),
    }

    KernelPage::switch();
    Paging::enable();
    Process::new("0:/blank");

    println!("Hello, World!");
    traceln!("Hello, World!");

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}
