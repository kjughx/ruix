use core::{
    hint,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct Lock {
    lock: AtomicUsize,
    id: Option<&'static str>, // For debugging,
}

impl Lock {
    pub const fn new(id: Option<&'static str>) -> Self {
        Self {
            lock: AtomicUsize::new(0),
            id,
        }
    }

    pub fn lock(&self) {
        while self.lock.load(Ordering::Acquire) == 1 {
            hint::spin_loop()
        }
        self.lock.store(1, Ordering::Release);
    }

    pub fn unlock(&self) {
        match self
            .lock
            .compare_exchange(1, 0, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => (),
            Err(_) => panic!("Poisoned lock: {:?}", self.id()),
        }
    }

    pub fn id(&self) -> &'static str {
        self.id.unwrap_or("UN-NAMED")
    }
}

//
// Pack read and write into the same value,
// usize::MAX means write-locked
// 0 means unlocked
// 1.. means read-locked and how many have it
pub struct RWLock {
    lock: AtomicUsize,
    id: Option<&'static str>,
}

impl RWLock {
    pub const fn new(id: Option<&'static str>) -> Self {
        Self {
            lock: AtomicUsize::new(0),
            id,
        }
    }

    pub fn rlock(&self) {
        lock(&self.lock, |a| a == usize::MAX, |a| a + 1);
    }

    pub fn wlock(&self) {
        lock(&self.lock, |a| a != 0, |_| usize::MAX)
    }

    pub fn runlock(&self) {
        unlock(&self.lock, |a| a != 0, |a| a - 1).expect("Poisoned read lock");
    }
    pub fn wunlock(&self) {
        unlock(&self.lock, |a| a == usize::MAX, |_| 0).expect("Poisoned write lock");
    }

    pub fn id(&self) -> Option<&'static str> {
        self.id
    }
}

#[inline]
fn lock<F, B>(atomic: &AtomicUsize, is_locked: B, lock: F)
where
    F: Fn(usize) -> usize,
    B: Fn(usize) -> bool,
{
    let mut current = atomic.load(Ordering::Acquire);
    loop {
        while is_locked(current) {
            hint::spin_loop();

            current = atomic.load(Ordering::Acquire);
            continue;
        }

        match atomic.compare_exchange_weak(
            current,
            lock(current),
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => return,
            Err(old) => current = old,
        }
    }
}

#[inline]
fn unlock<F, B>(atomic: &AtomicUsize, is_locked: B, op: F) -> Result<(), ()>
where
    F: Fn(usize) -> usize,
    B: Fn(usize) -> bool,
{
    let mut current = atomic.load(Ordering::Acquire);
    loop {
        if !is_locked(current) {
            return Err(());
        }

        match atomic.compare_exchange_weak(
            current,
            op(current),
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Ok(()),
            Err(old) => current = old,
        }
    }
}
