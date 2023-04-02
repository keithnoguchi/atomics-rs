//! A vanilla spinlock
//!
//! # Examples
//!
//! `#[cold]` brings the number down to 1.5s for 20m locks,
//! which is quite equivalent to the non-spin version in this
//! particular scenario, which is roughly 75ns/lock.
//!
//! ```
//! $ cargo +nightly run -qr
//! 20000000 locks in 1.488637562s
//! ```

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Instant;

use atomic_wait::{wait, wake_one};

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
        Self::lock_contended(&self.state);
        Guard { lock: self }
    }

    #[cold]
    fn lock_contended(state: &AtomicU32) {
        // spin in case there is no waiter.
        let mut spin_count = 0;
        while state.load(Relaxed) == 1 && spin_count < 100 {
            spin_count += 1;
            std::hint::spin_loop();
        }
        // see if I can win the contention.
        if state.compare_exchange(0, 1, Acquire, Relaxed).is_ok() {
            return;
        }

        // or just falls back to wait.
        while state.swap(2, Acquire) != 0 {
            wait(state, 2);
        }
    }
}

#[derive(Debug)]
pub struct Guard<'a, T> {
    lock: &'a Mutex<T>,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        // wake up only when there is a waiter, e.g. state 2.
        if self.lock.state.swap(0, Release) == 2 {
            wake_one(&self.lock.state);
        }
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

fn main() {
    let m = Mutex::new(0);
    #[cfg(features = "nightly-features")]
    std::hint::black_box(&m);

    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..5_000_000 {
                    *m.lock() += 1;
                }
            });
        }
    });
    let duration = start.elapsed();

    println!("{} locks in {:?}", 4 * 5_000_000, duration);
    assert_eq!(*m.lock(), 4 * 5_000_000);
}
