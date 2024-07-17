use crate::{
    boxed::Box,
    path::Path,
};

use core::any::Any;

mod filesystems;

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
        path: Path,
        mode: FileMode,
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

