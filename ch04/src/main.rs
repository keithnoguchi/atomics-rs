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

fn main() {
    static LOCK: SpinLock = SpinLock::new();
    static mut DATA: u8 = 0;
    let validator = thread::current();

    thread::scope(|s| {
        // workers.
        let increment = || {
            LOCK.lock();
            unsafe {
                DATA += 1;
            }
            LOCK.unlock();
            validator.unpark();
        };
        for _ in 0..100 {
            s.spawn(increment);
        }

        // a validator.
        let validate = || {
            LOCK.lock();
            let count = unsafe { DATA };
            LOCK.unlock();
            count
        };
        while validate() != 100 {
            println!("waiting...");
            thread::park_timeout(Duration::from_secs(1));
        }
        assert_eq!(validate(), 100);
    });
}
