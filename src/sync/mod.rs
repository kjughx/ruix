pub mod global;
pub mod mutex;
#[macro_use]
pub mod lock;
pub mod shared;

pub use global::Global;
pub use lock::RWLock;
pub use shared::{Shared, Weak};

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
