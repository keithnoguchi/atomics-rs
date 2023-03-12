//! An one-shot channel

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread::{self, Thread};

#[derive(Debug)]
pub struct Channel<T> {
    ready: AtomicBool,
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        println!("dropping...");
        if self.ready.load(Acquire) {
            unsafe { (*self.data.get_mut()).assume_init_drop() }
        }
    }
}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn split(&mut self) -> (Sender<T>, Receiver<T>) {
        // force drop the previous channel, if it's still
        // not split.
        *self = Self::new();
        (
            Sender {
                channel: self,
                receiver: thread::current(),
            },
            Receiver { channel: self },
        )
    }
}

#[derive(Debug)]
pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
    receiver: Thread,
}

impl<T> Sender<'_, T> {
    pub fn send(self, msg: T) {
        unsafe {
            (*self.channel.data.get()).write(msg);
        }
        self.channel.ready.store(true, Release);
        self.receiver.unpark();
    }
}

#[derive(Debug)]
pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Receiver<'_, T> {
    pub fn receive(self) -> T {
        while !self.channel.ready.swap(false, Acquire) {
            thread::park();
        }
        unsafe { (*self.channel.data.get()).assume_init_read() }
    }
}

fn main() {
    let mut channel = Channel::new();
    let (tx, rx) = channel.split();

    // a producer.
    thread::scope(|s| {
        s.spawn(|| {
            tx.send("Hello, World!".to_string());
        });

        // a consumer.
        let message = rx.receive();
        assert_eq!(message, String::from("Hello, World!"));
    });
}
