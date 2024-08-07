use core::arch::asm;

use crate::{
    cpu::InterruptFrame,
    io::outb,
    packed::{packed, Packed},
    syscall, traceln,
};

extern crate interrupts;

const MAX_INTERRUPTS: usize = 255;
// See docs
interrupts::interrupt_table!(255);

#[no_mangle]
fn interrupt_handler(i: u16, frame: *const InterruptFrame) {
    unsafe {
        traceln!("Interrupted: {}: {}", i, *frame);
    };

    if i == 13 {
        panic!("UNHANDLED PAGE FAULT");
    }
    outb(0x20, 0x20);
}

#[packed]
struct IDTDescriptor {
    offset_low: u16,
    selector: u16,
    unused: u8,
    type_attr: u8,
    offset_high: u16,
}

#[packed]
struct IDTRecord {
    limit: u16, // Size of table - 1
    base: u32,
}

impl IDTRecord {
    fn new(size: usize, base: *const IDTDescriptor) -> Self {
        Self {
            limit: size as u16 - 1,
            base: base as u32,
        }
    }
}

const KERNEL_CODE_SELECTOR: u16 = 0x08;
const _KERNEL_DATA_SELECTOR: u16 = 0x10;

impl IDTDescriptor {
    const fn none() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            unused: 0,
            type_attr: 0,
            offset_high: 0,
        }
    }

    fn new(cb: unsafe extern "C" fn()) -> Self {
        let addr = cb as *const () as u32;
        Self {
            offset_low: (addr & 0xFFFF) as u16,
            selector: KERNEL_CODE_SELECTOR,
            unused: 0,
            type_attr: 0xEE,
            offset_high: (addr >> 16) as u16,
        }
    }
}

static mut IDT_DESCRIPTORS: [IDTDescriptor; MAX_INTERRUPTS] =
    [IDTDescriptor::none(); MAX_INTERRUPTS];

static mut IDT_RECORD: IDTRecord = IDTRecord { limit: 0, base: 0 };

pub struct IDT;
impl IDT {
    #[allow(clippy::needless_range_loop)]
    pub fn load() {
        for i in 0..MAX_INTERRUPTS {
            Self::set(i, INTERRUPT_POINTER_TABLE[i]);
        }

        Self::set(0x80, syscall::entry_syscall);

        unsafe {
            IDT_RECORD = IDTRecord::new(
                MAX_INTERRUPTS * core::mem::size_of::<IDTDescriptor>(),
                IDT_DESCRIPTORS.as_ptr(),
            );
        }

        unsafe {
            asm!(r#"
            lidt [ebx]
        "#, in("ebx") core::ptr::addr_of!(IDT_RECORD)
            )
        }
    }

    pub fn set(i: usize, cb: unsafe extern "C" fn()) {
        if i >= MAX_INTERRUPTS {
            return;
        }
        unsafe { IDT_DESCRIPTORS[i] = IDTDescriptor::new(cb) };
    }
}
