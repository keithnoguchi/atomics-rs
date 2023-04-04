//! A Condvar and Mutex<T>

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::sync::atomic::AtomicU32;
use std::thread;

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
}

fn main() {
    let mutex = Mutex::new(0);
    let condvar = Condvar::new();

    let mut wakeups = 0;

    thread::scope(|s| {
        s.spawn(|| {
            dbg!(&mutex);
            dbg!(&condvar);
        });

        dbg!(&mutex);
        dbg!(&condvar);
    });

    assert!(wakeups < 10);
}
