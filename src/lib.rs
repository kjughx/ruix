#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(naked_functions)]
#![allow(internal_features)]
#![feature(ptr_internals)]
#![feature(dispatch_from_dyn)]
#![feature(coerce_unsized)]
#![feature(deref_pure_trait)]
#![feature(unsize)]

extern crate packed;

pub mod boxed;
pub mod disk;
pub mod fs;
pub mod heap;
pub mod io;
pub mod path;
pub mod start;
pub mod tty;
#[macro_use]
pub mod serial;
pub mod sync;

pub trait Packed: Sized {}

pub trait FromBytes: Packed {
    type Output;
    fn from_bytes(bytes: &[u8]) -> Self::Output;
}
