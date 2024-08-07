#![no_std]
#![no_main]

use ruix::{
    fs::VFS,
    gdt::GDT,
    idt::IDT,
    paging::{KernelPage, Paging},
    println, traceln,
    tty::Terminal,
};

#[no_mangle]
extern "C" fn kmain() {
    Terminal::init();

    GDT::load();

    IDT::load();

    VFS::resolve().expect("Resolve disks");

    KernelPage::switch();
    Paging::enable();

    println!("Hello, World!");
    traceln!("Hello, World!");
}
