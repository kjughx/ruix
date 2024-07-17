mod r#impl;
mod private;

use crate::{
    boxed::{Box, Dyn},
    disk::{Disk, Stream},
    fs::{FileDescriptor, FileMode, FileSystem, IOError},
    lock,
    path::Path,
    sync::Global,
    traceln,
};

use private::{FatDirectoryItem, FatH, FAT_DIRECTORY_ITEM_SIZE};
use r#impl::{FatDirectory, FatItem, FAT16_SIGNATURE};

pub struct Fat16 {
    disk_id: u32,
    header: FatH,
    root_dir: FatDirectory,
}

impl Fat16 {
    pub(self) fn new(disk_id: u32, header: FatH, root_dir: FatDirectory) -> Self {
        Self {
            disk_id,
            header,
            root_dir,
        }
    }

    pub fn resolve(disk: &Global<Disk>) -> Result<Dyn<dyn FileSystem>, FSError> {
        let (id, sector_size, header) = {
            let disk = lock!(disk);

            (disk.id, disk.sector_size, disk.stream().read_new::<FatH>())
        };

        if header.extended_header.signature != FAT16_SIGNATURE {
            return Err(FSError::NotOurFS);
        }

        let root_start = header.root();
        let size =
            header.primary_header.root_dir_entries as usize * FAT_DIRECTORY_ITEM_SIZE / sector_size;
        let root_dir = {
            let disk = lock!(disk);
            FatDirectory::new(&mut disk.stream(), root_start, size)
        };

        Ok(Dyn::new(Self::new(id, header, root_dir)))
    }

    fn root(&self) -> &FatDirectory {
        &self.root_dir
    }

    fn get_directory_entry(&self, stream: &mut dyn Stream, path: Path) -> Option<FatItem> {
        let mut iter = path.parts().into_iter();

        let root = self.root();
        let part = iter.next()?;

        {
            let mut current = root.find(stream, part)?;

            for next in iter {
                match current {
                    FatItem::Directory(ref dir) => current = dir.find(stream, next)?,
                    FatItem::File(_) => return None,
                }
            }
            Some(current)
        }
    }

    fn cluster_to_sector(&self, cluster: usize) -> usize {
        traceln!(
            "{} {}",
            self.root_dir.end,
            self.header.primary_header.sectors_per_cluster
        );
        self.root_dir.end + (cluster - 2) * self.header.primary_header.sectors_per_cluster as usize
    }
}

impl FileSystem for Fat16 {
    fn open(
        &self,
        stream: &mut dyn Stream,
        path: Path,
        _mode: FileMode,
    ) -> Result<Box<dyn FileDescriptor>, IOError> {
        let Some(entry) = self.get_directory_entry(stream, path) else {
            return Err(IOError::NoSuchFile);
        };

        let file = match entry {
            FatItem::Directory(_) => return Err(IOError::NotAFile),
            FatItem::File(f) => f,
        };

        let desc: Box<dyn FileDescriptor> = Box::new(FatFileDescriptor::new(self.disk_id, file));

        Ok(desc)
    }

    fn read(&self, _fd: Box<dyn FileDescriptor>) {
        todo!()
    }

    fn seek(&self) {
        todo!()
    }

    fn stat(&self) {
        todo!()
    }
    fn name(&self) -> &str {
        todo!()
    }

    fn close(&self) {
        todo!()
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

pub struct FatFileDescriptor {
    item: FatDirectoryItem,
    disk_id: u32,
    pos: usize,
}

impl FatFileDescriptor {
    fn new(disk_id: u32, item: FatDirectoryItem) -> Self {
        Self {
            disk_id,
            item,
            pos: 0,
        }
    }
}

use crate::fs::{FSError, SeekMode};
impl FileDescriptor for FatFileDescriptor {
    fn read(&self, size: usize, count: usize, buf: &mut [u8]) -> Result<(), IOError> {
        if buf.len() < size * count {
            return Err(IOError::InvalidArgument);
        }

        let disk = lock!(Disk::get(self.disk_id));
        let mut stream = disk.stream();

        let start_sector = disk
            .filesystem
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<Fat16>()
            .unwrap()
            .cluster_to_sector(self.item.first_cluster());

        assert!(self.pos == 0, "No support for reading twice yet");
        traceln!("{}", disk.sector_size);
        stream.seek_sector(start_sector);
        stream.read(buf, size * count);

        Ok(())
    }

    fn write(&mut self, _size: usize, _count: usize, _buf: &[u8]) -> Result<(), IOError> {
        todo!()
    }

    fn seek(&mut self, offset: isize, whence: SeekMode) {
        match whence {
            SeekMode::CurrentPosition => self.pos = (self.pos as isize + offset) as usize,
            SeekMode::EndOfFile => self.pos = (self.item.filesize as isize - offset) as usize,
            SeekMode::StartOfFile => self.pos = offset as usize,
        }
    }
}
