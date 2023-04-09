//! A spin lock
//!
//! # Examples
//!
//! ```
//! $ cargo +nightly r -qr
//! 20000000 locks in 1.783934859s (89.20ns/lock)
//! ```

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::time::Instant;

#[derive(Debug)]
pub struct Mutex<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T> Send for Mutex<T> where T: Send {}
unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> Guard<'_, T> {
        while self.state.swap(1, Acquire) != 0 {
            std::hint::spin_loop();
        }
        Guard { mutex: self }
    }
}

#[derive(Debug)]
pub struct Guard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.mutex.state.swap(0, Release);
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

fn main() {
    let m = Mutex::new(0);
    const WORKERS: usize = 4;
    const LOCKS: usize = 5_000_000;

    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..WORKERS {
            s.spawn(|| {
                for _ in 0..LOCKS {
                    *m.lock() += 1;
                }
            });
        }
    });
    let duration = start.elapsed();

    debug_assert_eq!(*m.lock(), WORKERS * LOCKS);
    println!(
        "{} locks in {:?} ({:.2}ns/lock)",
        WORKERS * LOCKS,
        duration,
        duration.as_nanos() as f64 / (WORKERS * LOCKS) as f64,
    );
}
