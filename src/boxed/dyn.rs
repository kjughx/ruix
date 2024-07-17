use crate::{
    heap::{alloc, free},
    traceln,
};

use core::{
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::Unique,
};

pub struct Dyn<T: ?Sized>(Unique<T>);
impl<T> Dyn<T> {
    pub fn new(x: T) -> Self {
        unsafe {
            let t_ptr = alloc::<T>(core::mem::size_of::<T>());
            t_ptr.write(x);
            Self(Unique::new_unchecked(t_ptr))
        }
    }

    pub fn as_ptr(&mut self) -> *mut T {
        self.0.as_ptr()
    }

    pub fn drop(self) {
        traceln!("Dropping Dyn");
        free::<T>(self.0.as_ptr())
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Dyn<U>> for Dyn<T> {}

impl<T: ?Sized> Deref for Dyn<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Dyn<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}
