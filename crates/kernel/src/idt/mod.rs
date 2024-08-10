use core::arch::asm;

use crate::{
    cpu::InterruptFrame,
    io::outb,
    packed::{packed, Packed},
    traceln,
};

extern crate interrupts;

#[repr(C, packed)]
pub struct InterruptEntry {
    pub ptr: unsafe extern "C" fn(),
    pub id: u16,
}

extern "C" {
    static START_ISRS: usize;
    static END_ISRS: usize;
}

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

#[inline(never)]
fn find_registered_isrs() -> &'static [InterruptEntry] {
    unsafe {
        let start: *const InterruptEntry = (&START_ISRS as *const usize) as *const InterruptEntry;
        let end: *const InterruptEntry = (&END_ISRS as *const usize) as *const InterruptEntry;

        let count = (end as usize - start as usize) / core::mem::size_of::<InterruptEntry>();

        &*core::ptr::slice_from_raw_parts(start, count)
    }
}

pub struct IDT;
impl IDT {
    #[allow(clippy::needless_range_loop)]
    pub fn load() {
        // Set defaults
        for i in 0..MAX_INTERRUPTS {
            Self::set(i, INTERRUPT_POINTER_TABLE[i]);
        }

        let entries = find_registered_isrs();
        for entry in entries {
            Self::set(entry.id as usize, entry.ptr)
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

    pub fn set(i: usize, cb: unsafe extern "C" fn()) {
        if i >= MAX_INTERRUPTS {
            return;
        }
        unsafe { IDT_DESCRIPTORS[i] = IDTDescriptor::new(cb) };
    }
}
