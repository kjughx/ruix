use crate::disk::Stream;
use crate::packed::{packed, Packed};

#[packed]
pub struct FatHeaderExt {
    drive_no: u8,
    win_nt_bit: u8,
    pub signature: u8,
    pub volume_id: u32,
    pub volume_id_string: [u8; 11],
    pub system_id_string: [u8; 8],
}

#[packed]
pub struct FatHeader {
    pub short_jmp_ins: [u8; 3],
    pub oem_identifier: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_copies: u8,
    pub root_dir_entries: u16,
    pub number_of_sectors: u16,
    pub media_type: u8,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub number_of_heads: u16,
    pub hidden_sectors: u32,
    pub sectors_big: u32,
}

#[packed]
pub struct FatH {
    pub primary_header: FatHeader,
    pub extended_header: FatHeaderExt,
}

impl FatH {
    pub fn root(&self) -> usize {
        let primary_header = self.primary_header;

        (primary_header.fat_copies as usize * primary_header.sectors_per_fat as usize
            + primary_header.reserved_sectors as usize)
            * 512
    }
}

pub const _FAT_HEADER_SIZE: usize = core::mem::size_of::<FatH>();

#[packed]
pub struct FatDirectoryItem {
    pub filename: [u8; 8],
    pub extension: [u8; 3],
    pub attributes: u8,
    pub reserved: u8,
    pub creation_time_ds: u8,
    pub creation_time: u16,
    pub creation_dat: u16,
    pub last_access: u16,
    pub extended_attributes: u16,
    pub last_mod_time: u16,
    pub last_mod_data: u16,
    pub first_cluster: u16,
    pub filesize: u32,
}

impl FatDirectoryItem {
    pub fn new(streamer: &mut dyn Stream) -> Self {
        let mut buf = [0; FAT_DIRECTORY_ITEM_SIZE];
        streamer.read(&mut buf, FAT_DIRECTORY_ITEM_SIZE);
        FatDirectoryItem::from(&buf)
    }

    pub fn first_cluster(&self) -> usize {
        self.first_cluster as usize
    }

    pub fn filename(&self) -> &str {
        core::str::from_utf8(&self.filename).unwrap_or("").trim()
    }

    pub fn extension(&self) -> &str {
        core::str::from_utf8(&self.extension).unwrap_or("").trim()
    }
}

pub const FAT_DIRECTORY_ITEM_SIZE: usize = core::mem::size_of::<FatDirectoryItem>();
impl From<&[u8; FAT_DIRECTORY_ITEM_SIZE]> for FatDirectoryItem {
    fn from(bytes: &[u8; FAT_DIRECTORY_ITEM_SIZE]) -> Self {
        unsafe { *(bytes.as_ptr() as *const FatDirectoryItem) }
    }
}
