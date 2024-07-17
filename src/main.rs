#![no_std]
#![no_main]

use core::hint;

use ruix::{
    disk::Disk,
    fs::Vfs,
    traceln,
    tty::{init_screen, print},
};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    init_screen();

    // Resolve the connected disks
    match Vfs::resolve(Disk::get_mut(0)) {
        Ok(()) => (),
        Err(_) => print("Could not resolve disk 0"),
    }

    print("Hello, World!");
    traceln!("Hello, World!");
    loop {
        hint::spin_loop()
    }
}
