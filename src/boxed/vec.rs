use crate::heap::{alloc, free, realloc};

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr::Unique,
};

#[derive(Clone)]
#[doc(hidden)]
pub struct Vec<T: Sized> {
    data: Unique<T>,
    cap: usize,
    len: isize,
    _marker: PhantomData<[T]>,
}

pub struct VecIter<'a, T> {
    vec: &'a Vec<T>,
    index: isize,
}

impl<'a, T> Iterator for VecIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.len {
            return None;
        }

        self.index += 1;
        Some(self.vec.index(self.index - 1))
    }
}

const DEFAULT_VEC_CAP: usize = 16;

impl<T: Copy> Vec<T> {
    pub fn new() -> Self {
        unsafe {
            let t_ptr = core::mem::transmute::<*mut u8, *mut T>(alloc(
                DEFAULT_VEC_CAP * core::mem::size_of::<T>(),
            ));

            Self {
                data: Unique::new_unchecked(t_ptr),
                cap: DEFAULT_VEC_CAP,
                len: 0,
                _marker: PhantomData,
            }
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        unsafe {
            let t_ptr =
                core::mem::transmute::<*mut u8, *mut T>(alloc(cap * core::mem::size_of::<T>()));

            Self {
                data: Unique::new_unchecked(t_ptr),
                cap,
                len: 0,
                _marker: PhantomData,
            }
        }
    }

    fn grow(&mut self) {
        unsafe {
            self.data = Unique::new_unchecked(core::mem::transmute::<*mut u8, *mut T>(realloc(
                self.data.as_ptr() as *mut u8,
                2 * self.cap,
            )));
        }
        self.cap *= 2;
    }

    pub fn push(&mut self, x: T) {
        if self.len as usize == self.cap {
            self.grow()
        }

        unsafe {
            *self.data.as_ptr().offset(self.len) = x;
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        let x = unsafe { *self.data.as_ptr().offset(self.len) };
        self.len -= 1;

        Some(x)
    }

    pub fn clear(&mut self) {
        self.len = 0;
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

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { &*core::ptr::slice_from_raw_parts(self.data.as_ptr(), self.cap) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *core::ptr::slice_from_raw_parts(self.data.as_ptr(), self.cap).cast_mut() }
    }
}

use core::fmt::Debug;
impl<T: Debug> Debug for Vec<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "[")?;
        for (i, el) in self.into_iter().enumerate() {
            write!(f, "\t{:#?}", el)?;

            if i != self.len as usize {
                writeln!(f, ",")?;
            }
        }
        writeln!(f, "]")?;

        Ok(())
    }
}

impl<T: Copy> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Sized> Drop for Vec<T> {
    fn drop(&mut self) {
        free(self.data.as_ptr())
    }
}

impl<T> Index<isize> for Vec<T> {
    type Output = T;
    fn index(&self, index: isize) -> &Self::Output {
        let index = if index < 0 {
            self.len - index - 1
        } else {
            index
        };

        unsafe { self.data.as_ptr().offset(index).as_ref().unwrap() }
    }
}

impl<T> IndexMut<isize> for Vec<T> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let index = if index < 0 {
            self.len - index - 1
        } else {
            index
        };
        unsafe { self.data.as_ptr().offset(index).as_mut().unwrap() }
    }
}

impl<T: Copy> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for element in iter {
            vec.push(element);
        }

        vec
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = VecIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        VecIter {
            vec: self,
            index: 0,
        }
    }
}
