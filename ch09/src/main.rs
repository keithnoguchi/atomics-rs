//! A mutex

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;

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
            wait(&self.state, 1);
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
        self.mutex.state.store(0, Release);
        wake_one(&self.mutex.state);
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

fn wait(a: &AtomicU32, expected: u32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            a as *const AtomicU32,
            libc::FUTEX_WAIT,
            expected,
        );
    }
}

fn wake_one(a: &AtomicU32) {
    unsafe {
        libc::syscall(libc::SYS_futex, a as *const AtomicU32, libc::FUTEX_WAKE, 1);
    }
}

fn main() {
    let counter = Mutex::new(0);

    thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| {
                for _ in 0..1000 {
                    *counter.lock() += 1;
                }
            });
        }
    });

    assert_eq!(*counter.lock(), 10 * 1000);
}
