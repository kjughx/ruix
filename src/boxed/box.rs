use crate::heap::{alloc, free};

use core::{
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::Unique,
};

pub struct Box<T: ?Sized>(Unique<T>);
impl<T> Box<T> {
    pub fn new(x: T) -> Self {
        unsafe {
            let t_ptr = alloc::<T>(core::mem::size_of::<T>());
            t_ptr.write(x);
            Self(Unique::new_unchecked(t_ptr))
        }
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        free::<T>(self.0.as_ptr())
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Box<U>> for Box<T> {}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}
