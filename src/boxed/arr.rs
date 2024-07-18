use crate::{
    heap::{alloc, free},
    traceln,
};

use core::{
    ops::{Index, IndexMut},
    ptr::Unique,
    slice::IterMut,
};

#[derive(Clone)]
#[doc(hidden)]
pub struct Array<T> {
    data: Unique<T>,
    cap: usize,
}

impl<T> Array<T> {
    pub fn new(cap: usize) -> Self {
        traceln!("Creating DynArray with {} capacity", cap);
        unsafe {
            let t_ptr =
                core::mem::transmute::<*mut u8, *mut T>(alloc(cap * core::mem::size_of::<T>()));

            core::ptr::write_bytes(t_ptr, 0, cap);

            Self {
                data: Unique::new_unchecked(t_ptr),
                cap,
            }
        }
    }

    pub fn free(&mut self) {
        free(self.data.as_ptr())
    }

    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            core::ptr::slice_from_raw_parts(self.data.as_ptr(), self.cap)
                .as_ref()
                .unwrap()
        }
    }
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            core::ptr::slice_from_raw_parts(self.data.as_ptr(), self.cap)
                .cast_mut()
                .as_mut()
                .unwrap()
        }
    }
}

impl<T> Index<usize> for Array<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.data.as_ptr().add(index).as_ref().unwrap() }
    }
}

impl<T> IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.data.as_ptr().add(index).as_mut().unwrap() }
    }
}

pub struct ArrIter<'a, T> {
    arr: &'a Array<T>,
    index: usize,
}

impl<'a, T> Iterator for ArrIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.arr.cap {
            return None;
        }

        self.index += 1;
        Some(self.arr.index(self.index - 1))
    }
}

impl<'a, T> IntoIterator for &'a Array<T> {
    type Item = &'a T;
    type IntoIter = ArrIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        ArrIter {
            arr: self,
            index: 0,
        }
    }
}

impl<'a, T> IntoIterator for &'a mut Array<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().iter_mut()
    }
}
