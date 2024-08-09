#![no_std]
#![no_main]

use kernel::{
    fs::VFS,
    gdt::GDT,
    idt::IDT,
    paging::{KernelPage, Paging},
    println,
    process::Process,
    tty::Terminal,
};

#[no_mangle]
extern "C" fn kmain() {
    Terminal::init();
    println!("Booting ruix v0.0.1");

    GDT::load();

    IDT::load();

    VFS::resolve().expect("Resolve disks");

    KernelPage::switch();
    Paging::enable();

    let process = Process::new("0:/SHELL").unwrap();

    Process::exec(process);
}
