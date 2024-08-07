use core::{
    cmp::{max, min},
    marker::PhantomData,
};

use private::{
    Header, PHeader, ELF_CLASS_32, ELF_CLASS_NONE, ELF_DATA_2LSB, ELF_DATA_NONE, ELF_SIGNATURE,
};

use crate::{
    boxed::Array,
    fs::{FileMode, VFS},
    path::Path,
    string::Str,
    traceln, FromBytes, ReinterpretBytes,
};

mod private;

pub struct Elf {
    filename: Str,
    file: Array<u8>,
    vbase: usize,
    vend: usize,
    pbase: usize,
    pend: usize,
    _marker: PhantomData<[u8]>,
}

impl Elf {
    pub fn load(filename: &str) -> Result<Self, super::Error> {
        let fd = VFS::open(Path::new(filename), FileMode::ReadOnly).unwrap();
        let mut file = fd.read_all().unwrap();
        let header = Header::from_bytes(&file[..Header::size()]);

        let ptr = file.as_ptr() as usize;

        if !Self::validate(header) {
            return Err(super::Error::BadFormat);
        }

        let (vbase, vend, pbase, pend) = {
            let mut offset = header.e_phoff;
            let pheader = PHeader::reinterpret_mut(&mut file[offset..offset + PHeader::size()]);
            offset += PHeader::size();
            // This is resevered for us to do
            pheader.p_paddr = ptr + pheader.p_offset;

            let mut vbase = pheader.p_vaddr;
            let mut vend = pheader.p_vaddr + pheader.p_memsz;
            let mut pbase = pheader.p_offset;
            let mut pend = pheader.p_offset + pheader.p_memsz;

            for _ in 1..header.e_phnum {
                let pheader = PHeader::reinterpret_mut(&mut file[offset..offset + PHeader::size()]);
                offset += PHeader::size();
                traceln!("{}", offset);
                // This is resevered for us to do
                pheader.p_paddr = ptr + pheader.p_offset;

                vbase = min(vbase, pheader.p_vaddr);
                vend = max(vend, pheader.p_vaddr + pheader.p_memsz);

                pbase = min(pbase, pheader.p_offset);
                pend = max(pend, pheader.p_offset + pheader.p_memsz);
            }
            (vbase, vend, pbase, pend)
        };

        Ok(Self {
            filename: Str::from(filename),
            file,
            vbase,
            vend,
            pbase: ptr + pbase,
            pend: ptr + pend,
            _marker: PhantomData,
        })
    }

    fn validate(header: Header) -> bool {
        let signature = &header.e_ident[0..4];
        let class = header.e_ident[4];
        let data = header.e_ident[5];

        signature == ELF_SIGNATURE
            && (data == ELF_DATA_NONE || data == ELF_DATA_2LSB)
            && (class == ELF_CLASS_NONE || class == ELF_CLASS_32)
            && header.e_phoff > 0
    }

    pub fn vbase(&self) -> *const () {
        self.vbase as *const ()
    }
    pub fn vend(&self) -> *const () {
        self.vend as *const ()
    }
    pub fn pbase(&self) -> *const () {
        self.pbase as *const ()
    }
    pub fn pend(&self) -> *const () {
        self.pend as *const ()
    }

    pub fn entry_point(&self) -> usize {
        Header::reinterpret(&self.file[0..Header::size()]).e_entry
    }

    pub fn pheaders(&self) -> Array<&PHeader> {
        let header = Header::reinterpret(&self.file[0..Header::size()]);
        let mut arr = Array::new(header.e_phnum as usize);
        for i in 0..header.e_phnum as usize {
            let offset = header.e_phoff + i * PHeader::size();
            arr[i] = PHeader::reinterpret(&self.file[offset..offset + PHeader::size()]);
        }

        arr
    }

    pub fn free(&mut self) {
        self.file.free()
    }
}
