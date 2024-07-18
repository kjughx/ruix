use core::arch::asm;

use crate::{cpu::InterruptFrame, io::outb, packed::{packed, Packed}, traceln};

extern crate interrupts;

const MAX_INTERRUPTS: usize = 128;

// See docs
interrupts::interrupt_table!(128);

#[no_mangle]
fn interrupt_handler(i: u16, frame: *const InterruptFrame) {
    unsafe {
        traceln!("Interrupted: {}: {:?}", i, *frame);
    };
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
    fn new(cb: *const u32) -> Self {
        let addr = cb as u32;
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
            let cb = INTERRUPT_POINTER_TABLE[i] as *const u32;
            Self::_set(i, cb);
        }

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

    pub fn _set(i: usize, cb: *const u32) {
        unsafe { IDT_DESCRIPTORS[i] = IDTDescriptor::new(cb) };
    }

    pub fn set(i: usize, cb: fn(InterruptFrame)) {
        Self::_set(i, cb as *const u32)
    }
}
