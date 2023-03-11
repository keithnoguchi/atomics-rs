//! A *safe* spin lock

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
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

fn main() {
    let data = &SpinLock::new(Vec::<char>::new());
    println!("{data:?}");

    thread::scope(|s| {
        for id in 0..5 {
            s.spawn(move || {
                for _ in 0..20 {
                    let data = data.lock();
                    println!("worker{id}: {data:?}");
                }
            });
        }
    });
    println!("{data:?}");
}
