use crate::{boxed::Array, disk::Stream, traceln};
use core::mem;

use super::private::{FatDirectoryItem, FAT_DIRECTORY_ITEM_SIZE};

pub(super) const FAT16_SIGNATURE: u8 = 0x29;
const _FAT16_ENTRY_SIZE: u16 = 0x02;
const _FAT16_BAD_SECTOR: u16 = 0xFF7;
const _FAT16_UNUSED: u8 = 0xE0;

const _FAT_FILE_READ_ONLY: u8 = 1 << 0;
const _FAT_FILE_HIDDEN: u8 = 1 << 1;
const _FAT_FILE_SYSTEM: u8 = 1 << 2;
const _FAT_FILE_VOLUME_LABEL: u8 = 1 << 3;
const FAT_FILE_SUBDIRECTORY: u8 = 1 << 4;
const _FAT_FILE_ARCHIVED: u8 = 1 << 5;
const _FAT_FILE_DEVICE: u8 = 1 << 6;
const _FAT_FILE_RESERVERED: u8 = 1 << 7;

pub(super) struct FatDirectory {
    items: Array<FatDirectoryItem>,
    pub _total: u32,
    pub _start: usize,
    pub end: usize,
}

impl FatDirectory {
    pub fn new(stream: &mut dyn Stream, start: usize, count: usize) -> Self {
        stream.seek(start);
        let total = Self::get_total_items(stream);

        let mut items = Array::new(count);
        for i in 0..total as isize {
            let item = FatDirectoryItem::new(stream);
            items[i] = item;
        }

        Self {
            items,
            _total: total,
            _start: start,
            end: start + count * FAT_DIRECTORY_ITEM_SIZE,
        }
    }

    fn get_total_items(stream: &mut dyn Stream) -> u32 {
        let pos = stream.pos(); // We have to rewind when done

        const SIZE: usize = mem::size_of::<FatDirectoryItem>();
        let mut buf: [u8; SIZE] = [0; SIZE];
        let mut count = 0;
        loop {
            stream.read(&mut buf, SIZE);
            match buf[0] {
                0 => break,
                0xE5 => continue,
                _ => count += 1,
            }
        }

        stream.seek(pos);
        count
    }

    pub fn find(&self, stream: &mut dyn Stream, name: &str) -> Option<FatItem> {
        traceln!();
        for item in self.items.into_iter() {
            traceln!("{}", item.filename());
            if item.filename() == name {
                return Some(FatItem::new(stream, item));
            }
        }

        None
    }
}

pub(super) enum FatItem {
    Directory(FatDirectory),
    File(FatDirectoryItem),
}

impl FatItem {
    pub fn new(stream: &mut dyn Stream, item: &FatDirectoryItem) -> Self {
        match item.attributes {
            FAT_FILE_SUBDIRECTORY => {
                let size = FatDirectoryItem::size(stream);
                FatItem::Directory(FatDirectory::new(stream, item.first_cluster(), size))
            }
            _ => FatItem::File(*item),
        }
    }
}
