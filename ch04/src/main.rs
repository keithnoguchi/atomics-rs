//! An unsafe spin lock

use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> &mut T {
        while self.locked.swap(true, Acquire) {
            std::hint::spin_loop();
        }
        unsafe { &mut *self.value.get() }
    }

    /// Safety: `&mut T` from `lock()` must be gone!
    pub unsafe fn unlock(&self) {
        self.locked.store(false, Release);
    }
}

fn main() {
    let validator = thread::current();
    let state = SpinLock::new(String::new());

    thread::scope(|s| {
        // workers.
        let work = || {
            for _ in 0..20 {
                let value = state.lock();
                value.push('!');
                unsafe {
                    state.unlock();
                }
            }
            validator.unpark();
        };
        for _ in 0..5 {
            s.spawn(work);
        }

        // a validator.
        loop {
            let value = state.lock();
            if value.len() == 100 {
                unsafe { state.unlock(); }
                break;
            }
            unsafe {
                state.unlock();
            };
            thread::park_timeout(Duration::from_secs(1));
        }
        // wait a bit before double checking.
        thread::sleep(Duration::from_millis(1));
        let value = state.lock();
        assert_eq!(value.len(), 100);
        unsafe {
            state.unlock();
        }
    });
}
