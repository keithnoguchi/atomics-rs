//! A Mutex<T> with spin and wait
//!
//! # Examples
//!
//! The spin and wait is 10% better time in this particular scenario.
//!
//! ```
//! cargo +nightly run -qr
//! 20000000 locks in 1.318941107s (65ns/lock)
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

    #[inline]
    pub fn lock(&self) -> Guard<'_, T> {
        if self.state.swap(1, Acquire) != 0 {
            // contended :(
            Self::lock_contended(&self.state);
        }
        Guard { lock: self }
    }

    #[cold]
    fn lock_contended(state: &AtomicU32) {
        let mut spins = 0;

        // spin 100 times while there is no waiter.
        while state.load(Relaxed) == 1 && spins < 100 {
            spins += 1;
            std::hint::spin_loop();
        }

        // lock it!
        if state.compare_exchange(0, 1, Acquire, Relaxed).is_ok() {
            return;
        }

        // or, wait.
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
        if self.lock.state.swap(0, Release) != 1 {
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
