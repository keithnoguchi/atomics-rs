//! A *safe* one-shot channel through panic
#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;

const EMPTY: u8 = 0;
const WRITING: u8 = 1;
const READY: u8 = 2;
const READING: u8 = 3;

#[derive(Debug)]
pub struct Channel<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    state: AtomicU8,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self {
            msg: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicU8::new(EMPTY),
        }
    }
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_ready(&self) -> bool {
        self.state.load(Relaxed) == READY
    }

    /// # Panics
    ///
    /// It panics with multiple sends.
    pub fn send(&self, msg: T) {
        if self
            .state
            .compare_exchange(EMPTY, WRITING, Relaxed, Relaxed)
            .is_err()
        {
            panic!("it's already in use");
        }
        unsafe {
            (*self.msg.get()).write(msg);
        }
        self.state.store(READY, Release);
    }

    /// # Panics
    ///
    /// It panics when there is no message.
    pub fn recv(&self) -> T {
        if self
            .state
            .compare_exchange(READY, READING, Acquire, Relaxed)
            .is_err()
        {
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
