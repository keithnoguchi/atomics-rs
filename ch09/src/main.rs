//! A Condvar and Mutex<T>
//!
//! # Examples
//!
//! ```
//! $ cargo run -qr
//! wakeups = 16
//! ```

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Duration;

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

    pub fn wait<'a, T>(&self, guard: Guard<'a, T>) -> Guard<'a, T> {
        let counter = self.counter.load(Relaxed);
        let mutex = guard.lock;
        drop(guard); // unlocks the mutex through Drop here.
        wait(&self.counter, counter); // wait if no notification happens.
        mutex.lock()
    }

    pub fn notify_one(&self) {
        self.counter.fetch_add(1, Relaxed);
        wake_one(&self.counter);
    }

    pub fn notify_all(&self) {
        self.counter.fetch_add(1, Relaxed);
        wake_all(&self.counter);
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

    pub fn lock(&self) -> Guard<T> {
        while self.state.swap(1, Acquire) != 0 {
            wait(&self.state, 1)
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
    let mutex = Mutex::new(0);
    let condvar = Condvar::new();

    let mut wakeups = 0;

    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..5 {
                    *mutex.lock() += 10;
                    thread::sleep(Duration::from_nanos(10));
                    condvar.notify_one();
                }
            });
        }

        let mut m = mutex.lock();
        while *m < 200 {
            m = condvar.wait(m);
            wakeups += 1;
        }
    });

    println!("wakeups = {wakeups}");
    assert!(wakeups < 25); // just in case of the thundering hurd issue
}
