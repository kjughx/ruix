#![no_std]
#![no_main]

use core::hint::spin_loop;

use kernel::{
    fs::VFS,
    platform::{Platform, Process},
    println,
    tty::Terminal,
};

#[no_mangle]
extern "C" fn kmain() {
    Terminal::init();
    println!("Booting ruix v0.0.1");

    Platform::init();
    VFS::resolve().expect("Resolve disks");

    Process::exec_new("0:/SHELL").unwrap();

    loop {
        spin_loop()
    }
}
