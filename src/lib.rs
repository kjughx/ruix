#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(naked_functions)]


pub mod start;
pub mod tty;
pub mod types;
pub mod io;
#[macro_use]
pub mod serial;
