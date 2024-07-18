use core::fmt::Debug;

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

impl Debug for InterruptFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ip = self.ip;
        let flags = self.flags;
        write!(f, r#" ip: 0x{:x}, flags: 0b{:b}"#, ip, flags)
    }
}
