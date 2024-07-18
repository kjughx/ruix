use crate::{
    boxed::Box,
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

pub enum FileMode {
    ReadOnly,
}

pub enum SeekMode {
    StartOfFile,
    CurrentPosition,
    EndOfFile,
}

pub trait FileSystem {
    fn open(
        &self,
        stream: &mut dyn Stream,
        path: Path,
        _mode: FileMode,
    ) -> Result<Box<dyn FileDescriptor>, IOError>;
    fn read(&self, fd: Box<dyn FileDescriptor>);
    fn seek(&self);
    fn stat(&self);
    fn close(&self);
    fn name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

pub trait FileDescriptor {
    fn read(&self, size: usize, count: usize, buf: &mut [u8]) -> Result<(), IOError>;
    fn write(&mut self, size: usize, count: usize, buf: &[u8]) -> Result<(), IOError>;
    fn seek(&mut self, offset: isize, whence: SeekMode);
}

pub struct Vfs;
impl Vfs {
    pub fn resolve(disk: &mut Global<Disk>) -> Result<(), FSError> {
        match Fat16::resolve(disk) {
            Ok(fs) => {
                disk.with_wlock(|disk| disk.register_filesystem(fs));
                return Ok(());
            }
            Err(FSError::NotOurFS) => (),
            Err(e) => Err(e)?,
        }

        Err(FSError::FSNotFound)
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
