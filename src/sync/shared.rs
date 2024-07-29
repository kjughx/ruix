use super::lock::RWLock;
use crate::heap::{alloc, free};

use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};

// This struct owns the value
struct SharedInner<T: Sized> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    rwlock: RWLock,
    data: T,
}

impl<T> SharedInner<T> {
    fn new(t: T) -> Self {
        Self {
            strong: AtomicUsize::new(1),
            weak: AtomicUsize::new(1),
            rwlock: RWLock::new(None),
            data: t,
        }
    }
}

unsafe impl<T: Sized + Sync + Send> Send for SharedInner<T> {}
unsafe impl<T: Sized + Sync + Send> Sync for SharedInner<T> {}

// An atomically reference-counted mutually exclusive accessed pointer to
// a value
pub struct Shared<T>(NonNull<SharedInner<T>>);

impl<T> Shared<T> {
    pub fn new(t: T) -> Self {
        let inner = unsafe {
            let x = SharedInner::new(t);
            let t_ptr = alloc::<SharedInner<T>>(core::mem::size_of::<SharedInner<T>>());
            t_ptr.write(x);
            NonNull::new(t_ptr).expect("non-null pointer")
        };

        Self(inner)
    }

    fn free(&mut self) {
        free(self.0.as_ptr())
    }

    fn inner(&self) -> &SharedInner<T> {
        unsafe { self.0.as_ref() }
    }

    fn inner_mut(&mut self) -> &mut SharedInner<T> {
        unsafe { self.0.as_mut() }
    }

    pub fn with_wlock<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let inner = self.inner_mut();
        inner.rwlock.wlock();
        f(&mut inner.data);
        inner.rwlock.wunlock()
    }

    pub fn with_rlock<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        let inner = self.inner();
        inner.rwlock.rlock();
        let r = f(&inner.data);
        inner.rwlock.runlock();
        r
    }

    #[inline]
    fn from_inner(ptr: NonNull<SharedInner<T>>) -> Self {
        Self(ptr)
    }

    pub fn weak(this: &Self) -> Weak<T> {
        this.inner().weak.fetch_add(1, Ordering::Relaxed);
        Weak(this.0)
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        let inner = self.inner();
        inner.strong.fetch_add(1, Ordering::Relaxed);
        Self::from_inner(self.0)
    }
}

pub struct Weak<T>(NonNull<SharedInner<T>>);

impl<T> Default for Weak<T> {
    fn default() -> Self {
        unsafe { Weak(NonNull::new_unchecked(0x3FFFFFFF as *mut SharedInner<T>)) }
    }
}

impl<T> Weak<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        let ptr = self.0.as_ptr();
        let (weak, lock) = unsafe { (&(*ptr).weak, &(*ptr).rwlock) };

        if weak.fetch_sub(1, Ordering::Release) == 1 {
            lock.wlock();
            free(ptr);
        }
    }
}
