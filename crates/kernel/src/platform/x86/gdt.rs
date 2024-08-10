use super::process::task::tss;
use crate::packed::{packed, Packed};
use core::{arch::asm, ptr::addr_of};

const GDT_SEGMENTS: usize = 6;

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

static TSS: tss::TSS = tss::TSS::new(0x600000, 0x10);

static mut GDTS: [__GDT; GDT_SEGMENTS] = [
    GDT::new(0x00, 0x00, 0x00, 0x00).encode(),
    GDT::new(0x00, 0xFFFFFF, 0b10011011, 0b1100).encode(), // Kernel Code Segment
    GDT::new(0x00, 0xFFFFFF, 0b10010011, 0b1100).encode(), // Kernel Data Segment
    GDT::new(0x00, 0xFFFFFF, 0b11111011, 0b1100).encode(), // User Code Segment,
    GDT::new(0x00, 0xFFFFFF, 0b11110011, 0b1100).encode(), // User Data Segment,
    GDT::new(0x00, 0x000000, 0b00000000, 0b0000).encode(), // Task State Segment, (Un-initialized)
];

// @base: base address of segment
// @limit: size of segment (NOTE: only 20 bits of 32)
//
// @access:
//      P(7): Present bit. Allows an entry to refer to a valid segment. Must be set (1) for any valid segment.
//      DPL(6:5): Descriptor privilege level field. Contains the CPU Privilege level of the segment. 0 = highest privilege (kernel), 3 = lowest privilege (user applications).
//      S(4): Descriptor type bit. If clear (0) the descriptor defines a system segment (eg. a Task State Segment). If set (1) it defines a code or data segment.
//      E(3): Executable bit. If clear (0) the descriptor defines a data segment. If set (1) it defines a code segment which can be executed from.
//      DC(2): Direction bit/Conforming bit.
//         For data selectors: Direction bit.
//         If clear (0) the segment grows up.
//         If set (1) the segment grows down, i.e. the Offset has to be greater than the Limit.
//         For code selectors: Conforming bit.
//         If clear (0) code in this segment can only be executed from the ring set in DPL.
//         If set (1) code in this segment can be executed from an equal or lower privilege level.
//            For example, code in ring 3 can far-jump to conforming code in a ring 2 segment.
//            For example, code in ring 0 cannot far-jump to a conforming code segment where DPL is 2, while code in ring 2 and 3 can.
//            The DPL field represent the highest privilege level that is allowed to execute the segment.
//            Note that the privilege level remains the same, ie. a far-jump from ring 3 to a segment with a DPL of 2 remains in ring 3 after the jump.
//      RW(1): Readable bit/Writable bit.
//         For code segments: Readable bit. If clear (0), read access for this segment is not allowed. If set (1) read access is allowed. Write access is never allowed for code segments.
//         For data segments: Writeable bit. If clear (0), write access for this segment is not allowed. If set (1) write access is allowed. Read access is always allowed for data segments.
//      A(0): Accessed bit. The CPU will set it when the segment is accessed unless set to 1 in advance.
//         This means that in case the GDT descriptor is stored in read only pages and this bit is set to 0, the CPU trying to set this bit will trigger a page fault.
//         Best left set to 1 unless otherwise needed.
//
// @flags: (NOTE: only 4 bits of 8)
//           G(3): Granularity flag, indicates the size the Limit value is scaled by. If clear (0), the Limit is in 1 Byte blocks (byte granularity). If set (1), the Limit is in 4 KiB blocks (page granularity).
//           DB(2): Size flag. If clear (0), the descriptor defines a 16-bit protected mode segment. If set (1) it defines a 32-bit protected mode segment. A GDT can have both 16-bit and 32-bit selectors at once.
//           L(1): Long-mode code flag. If set (1), the descriptor defines a 64-bit code segment. When set, DB should always be clear. For any other type of segment (other code types or any data segment), it should be clear (0).
//           R(0): reserved
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
            GDTS[5] = GDT::new(
                addr_of!(TSS) as u32,
                core::mem::size_of::<tss::TSS>() as u32,
                0b10001001,
                0b1100,
            )
            .encode();
            GDT_POINTER.size = (GDT_SEGMENTS * core::mem::size_of::<__GDT>() - 1) as u16;
            GDT_POINTER.base = GDTS.as_ptr() as u32;
            asm!("lgdt [{0}]", in(reg) addr_of!(GDT_POINTER));
            asm!("mov ax, 0x28");
            asm!("ltr ax");
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
