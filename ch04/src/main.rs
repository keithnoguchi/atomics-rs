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
    static mut DATA: String = String::new();
    let validator = thread::current();

    thread::scope(|s| {
        // 5 workers.
        let work = || {
            for _ in 0..20 {
                LOCK.lock();
                unsafe {
                    DATA.push('!');
                }
                LOCK.unlock();
            }
            validator.unpark();
        };
        for _ in 0..5 {
            s.spawn(work);
        }

        // a validator.
        let validate = || {
            LOCK.lock();
            let len = unsafe { DATA.len() };
            LOCK.unlock();
            len
        };
        while validate() != 100 {
            println!("waiting...");
            thread::park_timeout(Duration::from_secs(1));
        }
        assert_eq!(validate(), 100);
    });
}
