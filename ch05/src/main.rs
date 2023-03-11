//! A *safe* one-shot channel through panic
#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;

#[derive(Debug)]
pub struct Channel<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    in_use: AtomicBool,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self {
            msg: UnsafeCell::new(MaybeUninit::uninit()),
            in_use: AtomicBool::new(false),
            ready: AtomicBool::new(false),
        }
    }
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Relaxed)
    }

    /// # Panics
    ///
    /// It panics with multiple sends.
    pub fn send(&self, msg: T) {
        if self.in_use.swap(true, Relaxed) {
            panic!("it's already in use");
        }
        unsafe {
            (*self.msg.get()).write(msg);
        }
        self.ready.store(true, Release);
    }

    /// # Panics
    ///
    /// It panics when there is no message.
    pub fn recv(&self) -> T {
        if !self.ready.load(Acquire) {
            panic!("there is no message");
        }
        unsafe { (*self.msg.get()).assume_init_read() }
    }
}

fn main() {
    let channel = Channel::new();
    let current = thread::current();

    thread::scope(|s| {
        // a writer.
        s.spawn(|| {
            channel.send("hello, world!");
            current.unpark();
        });
    });

    // a reader.
    if !channel.is_ready() {
        thread::park();
    }
    assert_eq!(channel.recv(), "hello, world!");
}
