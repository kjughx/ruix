use crate::heap::{alloc, free};

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut, Index},
    ptr::Unique,
    slice::IterMut,
};

#[derive(Clone)]
#[doc(hidden)]
pub struct Array<T> {
    data: Unique<T>,
    cap: usize,
    _marker: PhantomData<[T]>,
}

impl<T> Array<T> {
    pub fn new(cap: usize) -> Self {
        unsafe {
            let t_ptr =
                core::mem::transmute::<*mut u8, *mut T>(alloc(cap * core::mem::size_of::<T>()));

            core::ptr::write_bytes(t_ptr, 0, cap);

            Self {
                data: Unique::new_unchecked(t_ptr),
                cap,
                _marker: PhantomData,
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

impl<T> Deref for Array<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { &*core::ptr::slice_from_raw_parts(self.data.as_ptr(), self.cap) }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *core::ptr::slice_from_raw_parts(self.data.as_ptr(), self.cap).cast_mut() }
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

impl<T: Copy> From<&[T]> for Array<T> {
    fn from(value: &[T]) -> Self {
        let mut arr = Self::new(value.len());
        for (i, el) in value.iter().enumerate() {
            arr[i] = *el;
        }

        arr
    }
}

impl<T: Copy> FromIterator<T> for Array<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (_, upper) = iter.size_hint();

        let mut arr: Array<T> = Array::new(upper.unwrap());
        for (i, element) in iter.into_iter().enumerate() {
            arr[i] = element;
        }

        arr
    }
}
