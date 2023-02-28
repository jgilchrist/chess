use std::sync::{Condvar, Mutex};

/// A `LockLatch` starts as false and eventually becomes true. You can block
/// until it becomes true.
pub struct LockLatch {
    m: Mutex<bool>,
    v: Condvar,
}

#[allow(unused)]
impl LockLatch {
    #[inline]
    pub const fn new() -> Self {
        Self {
            m: Mutex::new(false),
            v: Condvar::new(),
        }
    }

    /// Block until latch is set.
    #[inline]
    pub fn wait(&self) {
        let mut guard = self.m.lock().unwrap();
        while !*guard {
            guard = self.v.wait(guard).unwrap();
        }
    }

    // Sets the lock to true and notifies any threads waiting on it.
    #[inline]
    pub fn set(&self) {
        let mut guard = self.m.lock().unwrap();
        *guard = true;
        self.v.notify_all();
    }
}
