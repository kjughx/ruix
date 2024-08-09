mod r#impl;
mod private;
use crate::{
    boxed::{Array, Box, Dyn},
    disk::{Disk, Offset, Sector, Stream},
    fs::{FileDescriptor, FileMode, FileStat, FileSystem, IOError},
    path::Path,
    sync::Global,
};
use core::cell::Cell;

use private::{FatDirectoryItem, FatH};
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

    fn root(&self) -> &FatDirectory {
        &self.root_dir
    }

    fn get_directory_entry(&self, stream: &mut dyn Stream, path: Path) -> Option<FatItem> {
        let mut iter = path.parts().into_iter();

        let root = self.root();
        let part = iter.next()?;

        let mut current = root.find(stream, part)?;

        for next in iter {
            match current {
                FatItem::Directory(ref dir) => current = dir.find(stream, next)?,
                FatItem::File(_) => return None,
            }
        }
        Some(current)
    }

    fn cluster_to_sector(&self, cluster: usize) -> usize {
        self.root_dir.end + (cluster - 2) * self.header.primary_header.sectors_per_cluster as usize
    }
}

impl FileSystem for Fat16 {
    fn resolve(disk: &mut Global<Disk>) -> Result<(), FSError> {
        let (id, header) =
            disk.with_rlock(|disk| -> (u32, FatH) { (disk.id, disk.stream().read_new::<FatH>()) });

        if header.extended_header.signature != FAT16_SIGNATURE {
            return Err(FSError::NotOurFS);
        }

        let root_start = header.root();

        let root_dir = disk.with_rlock(|disk| -> FatDirectory {
            FatDirectory::new(
                &mut disk.stream(),
                Offset(root_start),
                header.primary_header.root_dir_entries as usize,
            )
        });

        let fs = Dyn::new(Self::new(id, header, root_dir));
        disk.with_wlock(|disk| disk.register_filesystem(fs));

        Ok(())
    }

    fn open(
        &self,
        stream: &mut dyn Stream,
        path: Path,
        mode: FileMode,
    ) -> Result<Box<dyn FileDescriptor>, IOError> {
        let Some(entry) = self.get_directory_entry(stream, path) else {
            return Err(IOError::NoSuchFile);
        };

        let file = match entry {
            FatItem::Directory(_) => return Err(IOError::NotAFile),
            FatItem::File(f) => f,
        };

        let desc: Box<dyn FileDescriptor> =
            Box::new(FatFileDescriptor::new(self.disk_id, file, mode));

        Ok(desc)
    }

    fn name(&self) -> &'static str {
        "FAT16"
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

pub struct FatFileDescriptor {
    item: FatDirectoryItem,
    disk_id: u32,
    pos: Cell<usize>,
    mode: FileMode,
}

impl FatFileDescriptor {
    fn new(disk_id: u32, item: FatDirectoryItem, mode: FileMode) -> Self {
        Self {
            disk_id,
            item,
            pos: Cell::new(0),
            mode,
        }
    }
}

use crate::fs::{FSError, SeekMode};
impl FileDescriptor for FatFileDescriptor {
    fn read(&self, size: usize) -> Result<Array<u8>, IOError> {
        if size > self.stat().size {
            return Err(IOError::InvalidArgument);
        }

        let mut buf = Array::new(size);

        Disk::get_mut(self.disk_id).with_rlock(|disk| {
            let mut stream = disk.stream();

            if self.pos.get() == 0 {
                let fs = disk
                    .filesystem
                    .as_ref()
                    .unwrap()
                    .as_any()
                    .downcast_ref::<Fat16>()
                    .expect("A FAT16 filesystem");

                let start_sector = fs.cluster_to_sector(self.item.first_cluster());
                stream.seek_sector(Sector(start_sector));
            }

            stream.seek(Offset(self.pos.get()));
            stream.read(&mut buf, size);
            self.pos.set(stream.pos().0);
        });

        Ok(buf)
    }

    fn read_all(&self) -> Result<Array<u8>, IOError> {
        let size = self.stat().size;
        let mut buf = Array::new(size);

        Disk::get_mut(self.disk_id).with_rlock(|disk| {
            let mut stream = disk.stream();

            let fs = disk
                .filesystem
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<Fat16>()
                .expect("A FAT16 filesystem");

            let start_sector = fs.cluster_to_sector(self.item.first_cluster());

            stream.seek_sector(Sector(start_sector));
            stream.read(&mut buf, size);
        });

        Ok(buf)
    }

    fn write(&mut self, _size: usize, _count: usize, _buf: &[u8]) -> Result<(), IOError> {
        todo!()
    }

    fn seek(&self, offset: isize, whence: SeekMode) {
        match whence {
            SeekMode::CurrentPosition => self.pos.set((self.pos.get() as isize + offset) as usize),
            SeekMode::EndOfFile => self
                .pos
                .set((self.item.filesize as isize - offset) as usize),
            SeekMode::StartOfFile => self.pos.set(offset as usize),
        }
    }

    fn stat(&self) -> FileStat {
        let descriptor = self.as_any().downcast_ref::<FatFileDescriptor>().unwrap();
        let size = descriptor.item.filesize;
        let mode = descriptor.mode;
        FileStat {
            mode,
            size: size as usize,
        }
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}
