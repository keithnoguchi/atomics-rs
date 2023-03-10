//! A super slow, but working, spin lock.

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Duration;

pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange(false, true, Acquire, Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Release);
    }
}

static LOCK: SpinLock = SpinLock::new();

fn main() {
    static mut DATA: u8 = 0;
    let increment = || {
        LOCK.lock();
        unsafe {
            DATA += 1;
        }
        LOCK.unlock();
    };

    // workers.
    for _i in 0..100 {
        thread::spawn(increment);
    }

    // a validator.
    let get = || -> u8 {
        LOCK.lock();
        let result = unsafe { DATA };
        LOCK.unlock();
        result
    };
    while get() != 100 {
        thread::park_timeout(Duration::from_secs(1));
    }
    assert_eq!(get(), 100);
}
