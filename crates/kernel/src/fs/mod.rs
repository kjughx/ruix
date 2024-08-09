use crate::{
    boxed::{Array, Box},
    disk::{Disk, Stream},
    path::Path,
    sync::Global,
};

use core::any::Any;

mod filesystems;
use filesystems::fat16::Fat16;

#[derive(Debug)]
pub enum FSError {
    NotOurFS,
    FSNotFound,
}

#[derive(Debug)]
pub enum IOError {
    InvalidDisk,
    NoSuchFile,
    NoFS,
    NotAFile,
    InvalidArgument,
}

#[derive(Clone, Copy)]
pub enum FileMode {
    ReadOnly,
}

pub struct FileStat {
    pub mode: FileMode,
    pub size: usize,
}

pub enum SeekMode {
    StartOfFile,
    CurrentPosition,
    EndOfFile,
}

pub trait FileSystem {
    fn resolve(disk: &mut Global<Disk>) -> Result<(), FSError>
    where
        Self: Sized;
    fn open(
        &self,
        stream: &mut dyn Stream,
        path: Path,
        _mode: FileMode,
    ) -> Result<Box<dyn FileDescriptor>, IOError>;
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

pub trait FileDescriptor {
    fn read(&self, size: usize) -> Result<Array<u8>, IOError>;
    fn read_all(&self) -> Result<Array<u8>, IOError>;
    fn write(&mut self, size: usize, count: usize, buf: &[u8]) -> Result<(), IOError>;
    fn seek(&self, offset: isize, whence: SeekMode);
    fn stat(&self) -> FileStat;
    fn as_any(&self) -> &dyn Any;
}

pub struct VFS;
impl VFS {
    pub fn resolve() -> Result<(), FSError> {
        let disk0 = Disk::get_mut(0);
        <Fat16 as FileSystem>::resolve(disk0)
    }

    pub fn open(path: Path, mode: FileMode) -> Result<Box<dyn FileDescriptor>, IOError> {
        let Some(disk_id) = path.disk_id else {
            return Err(IOError::InvalidDisk);
        };

        Disk::get_mut(disk_id).with_rlock(|disk| -> Result<Box<dyn FileDescriptor>, IOError> {
            let Some(ref fs) = disk.filesystem else {
                return Err(IOError::NoFS);
            };

            let mut stream = disk.stream();

            fs.open(&mut stream, path, mode)
        })
    }
}
