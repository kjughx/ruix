use crate::boxed::{Array, Vec};

pub struct String(Vec<u8>);

impl String {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Default for String {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for String {
    fn from(value: &str) -> Self {
        Self(value.bytes().collect())
    }
}

impl<'a> From<&'a String> for &'a str {
    fn from(val: &'a String) -> Self {
        unsafe { core::str::from_utf8_unchecked(val.0.as_slice()) }
    }
}

pub struct Str(Array<u8>);
impl Str {
    pub fn new(size: usize) -> Self {
        Self(Array::new(size))
    }
}

impl Default for Str {
    fn default() -> Self {
        Self::new(0)
    }
}
