use crate::{
    packed::{packed, Packed},
    traceln,
};

type Word = usize;
type SWord = isize;
type Half = u16;
type Offset = usize;
type Addr = usize;
type Byte = u8;

// 0x7f E L F
pub(super) const ELF_SIGNATURE: [u8; 4] = [0x7f, b'E', b'L', b'F'];

pub(super) const ELF_CLASS_NONE: u8 = 0;
pub(super) const ELF_CLASS_32: u8 = 1;

pub(super) const ELF_DATA_NONE: u8 = 0;
pub(super) const ELF_DATA_2LSB: u8 = 1;

pub(super) const PF_X: usize = 0x01;
pub(super) const PF_W: usize = 0x02;
pub(super) const PF_R: usize = 0x04;

#[packed]
pub struct PHeader {
    pub(super) p_type: Word,
    pub(super) p_offset: Offset,
    pub(super) p_vaddr: Addr,
    pub(super) p_paddr: Addr,
    pub(super) p_filesz: Word,
    pub(super) p_memsz: Word,
    pub(super) p_flags: Word,
    pub(super) p_align: Word,
}

impl PHeader {
    pub fn is_exec(&self) -> bool {
        self.p_flags & PF_X != 0
    }
    pub fn is_writable(&self) -> bool {
        traceln!("{}", self.p_flags & PF_W == PF_W);
        self.p_flags & PF_W == PF_W
    }
    pub fn filesz(&self) -> Word {
        self.p_filesz
    }
    pub fn memsz(&self) -> Word {
        self.p_memsz
    }
    pub fn vaddr(&self) -> usize {
        self.p_vaddr
    }
    pub fn paddr(&self) -> usize {
        self.p_paddr
    }
}

#[packed]
pub(super) struct SHeader {
    sh_name: Word,
    sh_type: Word,
    sh_flags: Word,
    sh_addr: Addr,
    sh_offset: Offset,
    sh_size: Word,
    sh_link: Word,
    sh_info: Word,
    sh_addralign: Word,
    sh_entsize: Word,
}

#[packed]
pub(super) struct Header {
    pub(super) e_ident: [u8; 9],
    pub(super) unused: [u8; 7],
    pub(super) e_type: Half,
    pub(super) e_machine: Half,
    pub(super) e_version: Word,
    pub(super) e_entry: Addr,
    pub(super) e_phoff: Offset,
    pub(super) e_shoff: Offset,
    pub(super) e_flags: Word,
    pub(super) e_ehsize: Half,
    pub(super) e_phentsize: Half,
    pub(super) e_phnum: Half,
    pub(super) e_shentsize: Half,
    pub(super) e_shnum: Half,
    pub(super) e_shstrndx: Half,
}

// I guess an enum for the union
// #[packed]
// pub(super) struct Dyn {
//     d_tasg: SWord,
//     union {
//         d_val: Word,
//         d_ptr: Addr,
//     } d_un;
// }

#[packed]
pub(super) struct Sym {
    st_name: Word,
    st_value: Addr,
    st_size: Word,
    st_info: Byte,
    st_other: Byte,
    st_shndx: Half,
}
