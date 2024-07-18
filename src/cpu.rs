use core::fmt::Display;

use crate::packed::{packed, Packed};

const PROGRAM_VIRTUAL_ADDRESS: usize = 0x400000;
const USER_DATA_SEGMENT: usize = 0x23;
const USER_CODE_SEGMENT: usize = 0x1B;
const PROGRAM_VIRTUAL_STACK_START: usize = 0x3FF000;

#[allow(dead_code)]
pub struct Registers {
    edi: usize,
    esi: usize,
    ebp: usize,
    ebx: usize,
    edx: usize,
    ecx: usize,
    eax: usize,

    ip: usize,
    cs: usize,
    flags: usize,
    esp: usize,
    ss: usize,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            edi: 0,
            esi: 0,
            ebp: 0,
            ebx: 0,
            edx: 0,
            ecx: 0,
            eax: 0,

            ip: PROGRAM_VIRTUAL_ADDRESS,
            cs: USER_CODE_SEGMENT,
            flags: 0,
            esp: PROGRAM_VIRTUAL_STACK_START,
            ss: USER_DATA_SEGMENT,
        }
    }
}

#[packed]
pub struct InterruptFrame {
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub unused: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub ip: u32,
    pub cs: u32,
    pub flags: u32,
    pub sp: u32,
    pub ss: u32,
}

impl Display for InterruptFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let edi = self.edi;
        let esi = self.esi;
        let ebp = self.ebp;
        let ebx = self.ebx;
        let edx = self.edx;
        let ecx = self.ecx;
        let eax = self.eax;
        let ip = self.ip;
        let cs = self.cs;
        let flags = self.flags;
        let sp = self.sp;
        let ss = self.ss;
        write!(
            f,
            r#"
            edi: 0x{:08x}    esi: 0x{:08x}    ebp: 0x{:08x}
            ebx: 0x{:08x}    edx: 0x{:08x}
            ecx: 0x{:08x}    eax: 0x{:08x}

            flags: 0b{:08b}
            ip: 0x{:08x}     cs: 0x{:08x}
            sp: 0x{:08x}
            ss: 0x{:08x}
        "#,
            edi, esi, ebp, ebx, edx, ecx, eax, flags, ip, cs, sp, ss
        )
    }
}
