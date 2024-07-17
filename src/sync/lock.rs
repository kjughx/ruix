use core::{
    hint,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Lock {
    locked: AtomicBool,
    id: Option<&'static str>, // For debugging,
}

impl Lock {
    pub const fn new(id: Option<&'static str>) -> Self {
        Self {
            locked: AtomicBool::new(false),
            id,
        }
    }

    pub fn lock(&self) {
        while self.locked.load(Ordering::Acquire) {
            hint::spin_loop()
        }
        self.locked.store(true, Ordering::Release);
    }

    pub fn unlock(&self) {
        assert!(self.locked.load(Ordering::Acquire));
        // traceln!("Unlocking {}", self.id);
        self.locked.store(false, Ordering::Release);
    }

    pub fn id(&self) -> &'static str {
        self.id.unwrap_or("")
    }
}

#[macro_export]
macro_rules! lock {
    ($global:expr) => {
        $global.lock()
    };
}
