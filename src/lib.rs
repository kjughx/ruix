#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(naked_functions)]
#![allow(internal_features)]
#![feature(ptr_internals)]
#![feature(dispatch_from_dyn)]
#![feature(coerce_unsized)]
#![feature(deref_pure_trait)]
#![feature(unsize)]
#![allow(dead_code)]
extern crate global;
extern crate packed;

mod boot;
pub mod boxed;
pub mod cpu;
pub mod disk;
pub mod fs;
pub mod gdt;
pub mod heap;
pub mod idt;
pub mod io;
pub mod loader;
pub mod paging;
pub mod path;
pub mod process;
pub mod start;
pub mod string;
pub mod task;
pub mod tty;
#[macro_use]
pub mod serial;
pub mod sync;

pub trait Packed: Sized {}

pub trait FromBytes: Packed {
    type Output;
    fn from_bytes(bytes: &[u8]) -> Self::Output;
}
pub trait ReinterpretBytes: Packed {
    type Output;
    fn reinterpret(bytes: &[u8]) -> &Self::Output;
    fn reinterpret_mut(bytes: &mut [u8]) -> &mut Self::Output;
}

pub enum Error {
    InvalidArgument,
}
