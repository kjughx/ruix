use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    ptr::addr_of,
};

use super::lock::RWLock;

enum State<T, F = fn() -> T> {
    Uninit(F),
    Init(T),
    Poisoned,
}
pub struct Global<T, F = fn() -> T> {
    data: UnsafeCell<State<T, F>>,
    lock: RWLock,
}

unsafe impl<T, F: FnOnce() -> T> Sync for Global<T, F> {}

impl<T, F: FnOnce() -> T> Global<T, F> {
    pub const fn new(f: F, id: &'static str) -> Self {
        Self {
            data: UnsafeCell::new(State::Uninit(f)),
            lock: RWLock::new(Some(id)),
        }
    }

    pub unsafe fn force(&mut self) -> GlobalUnlocked<'_, T, F> {
        GlobalUnlocked::new(self)
    }

    /// # Safety: Unsafe because it needs manual unlock
    pub unsafe fn wlock(&mut self) -> GlobalUnlocked<'_, T, F> {
        self.lock.wlock();
        GlobalUnlocked::new(self)
    }

    fn rlock(&self) -> GlobalUnlocked<'_, T, F> {
        self.lock.rlock();
        GlobalUnlocked::new(self)
    }

    pub fn id(&self) -> &'static str {
        self.lock.id().unwrap()
    }

    pub fn with_wlock<S, U>(&mut self, f: S) -> U
    where
        S: FnOnce(&mut T) -> U,
    {
        let mut inner = unsafe { self.wlock() };
        let r = f(&mut inner);
        self.lock.wunlock();
        r
    }

    pub fn with_rlock<S, U>(&self, f: S) -> U
    where
        S: FnOnce(&T) -> U,
    {
        let inner = self.rlock();
        let r = f(&inner);
        self.lock.runlock();
        r
    }
}

pub struct GlobalUnlocked<'a, T: 'a, F: FnOnce() -> T = fn() -> T> {
    global: &'a Global<T, F>,
}

impl<'a, T: 'a, F: FnOnce() -> T> GlobalUnlocked<'a, T, F> {
    fn new(lock: &'a Global<T, F>) -> Self {
        Self { global: lock }
    }

    pub fn inner(&self) -> &T {
        let state = unsafe { &*self.global.data.get() };

        match state {
            State::Init(data) => data,
            State::Uninit(_) => {
                Self::force_init(self);
                let State::Init(t) = (unsafe { &*self.global.data.get() }) else {
                    unreachable!("State should be Init after forcing init")
                };
                t
            }
            State::Poisoned => panic!("Poisoned Global"),
        }
    }

    pub fn inner_mut(&mut self) -> &mut T {
        let state = unsafe { &mut *self.global.data.get() };
        match state {
            State::Init(t) => t,
            State::Uninit(_) => {
                Self::force_init(self);
                let State::Init(t) = (unsafe { &mut *self.global.data.get() }) else {
                    unreachable!()
                };
                t
            }
            State::Poisoned => panic!(),
        }
    }

    fn force_init(this: &Self) {
        let state = unsafe { &mut *this.global.data.get() };

        let State::Uninit(f) = core::mem::replace(state, State::Poisoned) else {
            unreachable!()
        };

        let data = f();

        unsafe { this.global.data.get().write(State::Init(data)) };
    }
}

impl<'a, T: 'a, F: FnOnce() -> T> Deref for GlobalUnlocked<'a, T, F> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        GlobalUnlocked::inner(self)
    }
}

impl<'a, T: 'a, F: FnOnce() -> T> DerefMut for GlobalUnlocked<'a, T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        GlobalUnlocked::inner_mut(self)
    }
}
