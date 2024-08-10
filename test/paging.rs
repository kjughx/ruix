#![no_std]
#![no_main]

use ruix::{
    gdt::GDT,
    idt::IDT,
    paging::{KernelPage, Paging, PAGE_IS_PRESENT},
    traceln,
};

#[no_mangle]
extern "C" fn kmain() {
    GDT::load();
    IDT::load();

    KernelPage::switch();

    unsafe {
        *(0x1000 as *mut u32) = 65;
    }

    let directory = KernelPage::get_mut();
    unsafe {
        directory.force().map(
            ruix::paging::Addr(0x4000),
            ruix::paging::Addr(0x1000),
            PAGE_IS_PRESENT,
        )
    };

    Paging::enable();

    unsafe { traceln!("{}", *(0x4000 as *const u32)) };
}
