#![no_std]
#![no_main]

use ruix::{
    gdt::GDT,
    idt::IDT,
    paging::{KernelPage, Paging, PAGE_IS_PRESENT},
    traceln,
};

/// echo "TESTING" | sudo tee /mnt/d/TEST

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

    let fd = Vfs::open(
        ruix::path::Path::new("0:/TEST"),
        ruix::fs::FileMode::ReadOnly,
    )
    .unwrap();

    let mut buf: [u8; 32] = [0; 32];
    fd.read(32, 1, &mut buf).unwrap();
    traceln!("{:?}", core::str::from_utf8(&buf));
}
