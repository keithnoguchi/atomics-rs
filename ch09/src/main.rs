//! A Condvar and Mutex<T>
//!
//! # Examples
//!
//! ```
//! cargo +nightly run -qr
//! 20000000 locks in 2.966500125s (148ns/lock)
//! ```

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Release};
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

    #[inline]
    pub fn lock(&self) -> Guard<'_, T> {
        while self.state.swap(1, Acquire) != 0 {
            wait(&self.state, 1);
        }
        Guard { lock: self }
    }
}

#[derive(Debug)]
pub struct Guard<'a, T> {
    lock: &'a Mutex<T>,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Release);
        wake_one(&self.lock.state);
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

const WORKERS: usize = 4;
const LOCKS: usize = 5_000_000;

fn main() {
    let m = Mutex::new(0);
    #[cfg(feature = "nightly-features")]
    std::hint::black_box(&m);

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

    println!(
        "{} locks in {:?} ({:?}/lock)",
        WORKERS * LOCKS,
        duration,
        duration / ((WORKERS * LOCKS) as u32),
    );
    assert_eq!(*m.lock(), WORKERS * LOCKS);
}
