//! A *safe* spin lock

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;

#[derive(Debug)]
struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> Guard<T> {
        while self
            .locked
            .compare_exchange(false, true, Acquire, Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
        Guard { lock: self }
    }
}

#[derive(Debug)]
struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

fn main() {
    let data = &SpinLock::new(vec![]);

    thread::scope(|s| {
        for id in 0..2 {
            s.spawn(move || {
                for _ in 0..2 {
                    data.lock().push(id);
                }
            });
        }
    });

    let data = data.lock();
    assert!(
        data.as_ref() == [0, 0, 1, 1]
            || data.as_ref() == [0, 1, 0, 1]
            || data.as_ref() == [0, 1, 1, 0]
            || data.as_ref() == [1, 0, 0, 1]
            || data.as_ref() == [1, 0, 1, 0]
            || data.as_ref() == [1, 1, 0, 0],
    );
}
