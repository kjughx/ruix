use crate::packed::{packed, Packed};

#[packed]
pub struct Tss {
    link: u32,
    esp0: u32, /* Kernel stack pointer */
    ss0: u32,  /* Kernel stack segment */
    esp1: u32,
    esp2: u32,
    ss2: u32,
    sr3: u32,
    eip: u32,
    eflags: u32,
    eax: u32,
    ecx: u32,
    edx: u32,
    ebx: u32,
    esp: u32,
    ebp: u32,
    esi: u32,
    edi: u32,
    es: u32,
    cs: u32,
    ss: u32,
    ds: u32,
    fs: u32,
    gs: u32,
    ldtr: u32,
    iopb: u32,
}

impl Tss {
    pub const fn new(esp: u32, ss: u32) -> Self {
        Self {
            link: 0,
            esp0: esp, /* Kernel stack pointer */
            ss0: ss,   /* Kernel stack segment */
            esp1: 0,
            esp2: 0,
            ss2: 0,
            sr3: 0,
            eip: 0,
            eflags: 0,
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            es: 0,
            cs: 0,
            ss: 0,
            ds: 0,
            fs: 0,
            gs: 0,
            ldtr: 0,
            iopb: 0,
        }
    }
}
