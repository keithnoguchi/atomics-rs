//! A Mutex<T> and Condvar
//!
//! # Examples
//!
//! ```
//! $ cargo +nightly run -qr
//! 17744 wakeups for 40000 notify_one() call (44.36%) in 529.856108ms
//! ```

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::{Duration, Instant};

use atomic_wait::{wait, wake_all, wake_one};

#[derive(Debug)]
pub struct Condvar {
    counter: AtomicU32,
}

impl Condvar {
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
        }
    }

    #[inline]
    pub fn notify_one(&self) {
        self.counter.fetch_add(1, Relaxed);
        wake_one(&self.counter);
    }

    #[inline]
    pub fn notify_all(&self) {
        self.counter.fetch_add(1, Relaxed);
        wake_all(&self.counter);
    }

    #[inline]
    pub fn wait<'a, T>(&self, guard: Guard<'a, T>) -> Guard<'a, T> {
        let counter = self.counter.load(Relaxed);
        let lock = guard.lock;
        drop(guard);
        wait(&self.counter, counter);
        lock.lock()
    }
}

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
        if self.state.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
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
const LOCKS: usize = 10_000;

fn main() {
    let m = Mutex::new(0);
    #[cfg(feature = "nightly-features")]
    std::hint::black_box(&m);

    let mut wakeups = 0;
    let cond = Condvar::new();
    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..WORKERS {
            s.spawn(|| {
                for _ in 0..LOCKS {
                    *m.lock() += 1;
                    thread::sleep(Duration::from_nanos(10));
                    cond.notify_one();
                }
            });
        }

        let mut lock = m.lock();
        while *lock != WORKERS * LOCKS {
            lock = cond.wait(lock);
            wakeups += 1;
        }
    });
    let duration = start.elapsed();

    println!(
        "{} wakeups for {} notify_one() call ({}%) in {:?}",
        wakeups,
        WORKERS * LOCKS,
        (wakeups as f32) / ((WORKERS * LOCKS) as f32) * 100.0,
        duration,
    );
    assert!(wakeups <= WORKERS * LOCKS + 10);
}
