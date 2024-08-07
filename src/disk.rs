use crate::FromBytes;

use crate::{
    boxed::{Dyn, Vec},
    fs::FileSystem,
    global::global,
    io::{insb, insw, outb},
    spinwhile,
    sync::Global,
};

#[derive(Clone, Copy)]
pub struct Sector(pub usize);
#[derive(Clone, Copy)]
pub struct Offset(pub usize);

const SECTOR_SIZE: usize = 512;

#[derive(Debug)]
pub enum IOError {
    Other,
}

pub enum DiskType {
    Real,
}

pub struct Disk {
    _disk_type: DiskType,
    pub sector_size: usize,
    pub id: u32,

    pub filesystem: Option<Dyn<dyn FileSystem>>,

    pos: usize,
}

impl Disk {
    pub fn new(disk_type: DiskType, sector_size: usize, id: u32) -> Self {
        Self {
            _disk_type: disk_type,
            sector_size,
            id,
            filesystem: None,
            pos: 0,
        }
    }

    pub fn get(_id: u32) -> &'static Global<Disk> {
        Disk0::get()
    }

    pub fn get_mut(_id: u32) -> &'static mut Global<Disk> {
        Disk0::get_mut()
    }

    pub fn register_filesystem(&mut self, fs: Dyn<dyn FileSystem>) {
        self.filesystem = Some(fs)
    }

    pub fn seek(&mut self, pos: Offset) {
        self.pos = pos.0;
    }

    pub fn stream(&self) -> Streamer {
        Streamer::new(self)
    }
}

global! {
    Disk0,
    Disk,
    Disk::new(DiskType::Real, SECTOR_SIZE, 0), "DISK0"
}

pub trait Stream {
    fn seek(&mut self, pos: Offset);
    fn seek_sector(&mut self, pos: Sector);
    fn pos(&self) -> Offset;
    fn read(&mut self, buf: &mut [u8], total: usize);
    fn write(&mut self) {
        todo!();
    }
    fn sector_size(&self) -> usize;
}

pub struct Streamer<'a> {
    pos: usize,
    disk: &'a Disk,
}

impl Stream for Streamer<'_> {
    fn seek(&mut self, pos: Offset) {
        self.pos = pos.0;
    }

    fn seek_sector(&mut self, sector: Sector) {
        self.pos = sector.0 * self.disk.sector_size;
    }

    fn pos(&self) -> Offset {
        Offset(self.pos)
    }

    fn read(&mut self, buf: &mut [u8], total: usize) {
        let mut bytes_read = 0;

        while bytes_read != total {
            let sector = self.pos / SECTOR_SIZE;
            let offset = self.pos % SECTOR_SIZE;
            let mut bytes_to_read = total - bytes_read;
            let mut local: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];

            if offset + bytes_to_read >= SECTOR_SIZE {
                // Read up to next sector boundary
                bytes_to_read -= (offset + bytes_to_read) - SECTOR_SIZE;
            }

            self.read_sector(sector as u32, &mut local);

            buf[bytes_read..(bytes_to_read + bytes_read)]
                .clone_from_slice(&local[offset..(offset + bytes_to_read)]);

            self.pos += bytes_to_read;
            bytes_read += bytes_to_read;
        }
    }

    fn sector_size(&self) -> usize {
        self.disk.sector_size
    }
}

impl<'a> Streamer<'a> {
    pub fn new(disk: &'a Disk) -> Self {
        Self { pos: 0, disk }
    }

    fn read_sector(&self, lba: u32, buf: &mut [u8; SECTOR_SIZE]) {
        outb(0x1F6, ((lba >> 24) | 0xE0) as u8);
        outb(0x1F2, 1);
        outb(0x1F3, (lba & 0xff) as u8);
        outb(0x1F4, (lba >> 8) as u8);
        outb(0x1F5, (lba >> 16) as u8);
        outb(0x1F7, 0x20);

        spinwhile!(insb(0x1F7) & 0x08 == 0);
        insw(0x1F7); // garbage

        for i in 0..(SECTOR_SIZE / 2) {
            let val = insw(0x1F0);
            buf[2 * i] = (val & 0xff) as u8;
            buf[2 * i + 1] = (val >> 8) as u8;
        }
    }

    pub fn read_new<T: FromBytes<Output = T> + Sized>(&mut self) -> T {
        let size = core::mem::size_of::<T>();

        let mut buf: Vec<u8> = Vec::with_capacity(size);

        self.read(buf.as_slice_mut(), size);
        T::from_bytes(buf.as_slice())
    }
}
