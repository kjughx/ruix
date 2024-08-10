extern crate syscall_macro;

#[cfg(feature = "kernel")]
pub use syscall_macro::{gen_syscalls, syscall};

pub use syscall_macro::syscalls;

#[syscalls]
extern "C" {
    pub fn exit(code: i32) -> usize;
}
