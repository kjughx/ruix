use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

pub struct Lock(AtomicBool);

impl Lock {
    fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    fn lock(&self) {
        while self.0.load(Ordering::Acquire) {}
        self.0.store(true, Ordering::Release);
    }
    fn unlock(&self) {
        assert!(self.0.load(Ordering::Acquire));
        self.0.store(false, Ordering::Release);
    }
}

pub struct Mutex<T> {
    data: UnsafeCell<T>,
    lock: Lock,
}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            lock: Lock::new(),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.lock.lock();
        MutexGuard::new(self)
    }
}

pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>,
}

impl<'a, T: 'a> MutexGuard<'a, T> {
    fn new(lock: &'a Mutex<T>) -> Self {
        Self { lock }
    }

    pub fn unlock(&self) {
        self.lock.lock.unlock();
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.unlock();
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}
