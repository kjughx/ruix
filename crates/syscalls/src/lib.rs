#![no_std]
#![feature(naked_functions)]
#![feature(asm_const)]

#[cfg(feature = "x86")]
pub mod x86;
#[cfg(feature = "x86")]
pub use x86::*;
