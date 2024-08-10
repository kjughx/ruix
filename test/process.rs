#![no_std]
#![no_main]

use ruix::{
    cpu,
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
extern "C" fn kmain() {
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
    let process = Process::new("0:/BLANK").unwrap();

    cpu::CPU::return_to_task(&process.task);
}
