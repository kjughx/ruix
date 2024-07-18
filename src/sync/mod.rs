pub mod global;
pub mod mutex;
#[macro_use]
pub mod lock;

pub use global::Global;
pub use lock::RWLock;

#[macro_export]
macro_rules! spinwhile {
    ($cond:expr) => {
        while $cond {}
    };
}

#[macro_export]
macro_rules! spinuntil {
    ($cond:expr) => {
        while !($cond) {}
    };
}
