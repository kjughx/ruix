use crate::packed::{packed, Packed};
use core::arch::asm;

const GDT_SEGMENTS: usize = 3;
#[packed]
struct __GDT {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    limit_flags: u8, // 4-bit flags << 4 | 4-bit limit_high
    base_high: u8,
}

const GDT_SIZE: usize = core::mem::size_of::<__GDT>();

impl From<&[u8; GDT_SIZE]> for __GDT {
    fn from(bytes: &[u8; GDT_SIZE]) -> Self {
        unsafe { *(bytes.as_ptr() as *const __GDT) }
    }
}

static GDTS: [GDT; GDT_SEGMENTS] = [
    GDT::new(0x00, 0x00, 0x00, 0x00),
    GDT::new(0x00, 0xFFFFFFFF, 0b10011010, 0b1100), // Kernel Code Segment
    GDT::new(0x00, 0xFFFFFFFF, 0b10010010, 0b1100), // Kernel Data Segment
];
static __GDTS: [__GDT; GDT_SEGMENTS] = [GDTS[0].encode(), GDTS[1].encode(), GDTS[2].encode()];

pub struct GDT {
    base: u32,
    limit: u32, // 20 bits
    access: u8,
    flags: u8,
}

#[packed]
struct GDTPointer {
    size: u16,
    base: u32,
}

static mut GDT_POINTER: GDTPointer = GDTPointer { size: 0, base: 0 };

impl GDT {
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        Self {
            base,
            limit,
            access,
            flags,
        }
    }

    pub fn load() {
        unsafe {
            GDT_POINTER.size = (GDT_SEGMENTS * core::mem::size_of::<__GDT>()) as u16;
            GDT_POINTER.base = __GDTS.as_ptr() as u32;
            asm!("lgdt [{0}]", in(reg) core::ptr::addr_of!(GDT_POINTER))
        }
    }

    const fn encode(&self) -> __GDT {
        let limit_low = (self.limit & 0xFFFF) as u16;
        let base_low = (self.base & 0xFFFF) as u16;
        let base_middle = ((self.base >> 16) & 0xFF) as u8;
        let access = self.access;
        let limit_flags = ((self.flags & 0x0f) << 4) | (((self.limit >> 16) & 0x0f) as u8);
        let base_high = ((self.base >> 24) & 0xFF) as u8;

        __GDT {
            limit_low,
            base_low,
            base_middle,
            access,
            limit_flags,
            base_high,
        }
    }
}
