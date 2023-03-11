//! A safe one-shot channel through types.
#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel::default());

    (
        Sender {
            channel: channel.clone(),
        },
        Receiver { channel },
    )
}

#[derive(Debug)]
pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Sender<T> {
    pub fn send(self, msg: T) {
        unsafe {
            (*self.channel.data.get()).write(msg);
        }
        self.channel.ready.store(true, Release);
    }
}

#[derive(Debug)]
pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Receiver<T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Relaxed)
    }

    /// # Panics
    ///
    /// It panics when there is no message on the channel yet.
    pub fn receive(self) -> T {
        if !self.channel.ready.swap(false, Acquire) {
            panic!("no message on the channel");
        }
        unsafe { (*self.channel.data.get()).assume_init_read() }
    }
}

#[derive(Debug)]
struct Channel<T> {
    ready: AtomicBool,
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self {
            ready: AtomicBool::new(false),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if self.ready.load(Acquire) {
            unsafe {
                (*self.data.get()).assume_init_drop();
            }
        }
    }
}

fn main() {
    let (tx, rx) = channel();
    let consumer = thread::current();

    thread::scope(|s| {
        // a producer.
        s.spawn(|| {
            tx.send(String::from("hello, world!"));
            consumer.unpark();
        });
    });

    // a consumer.
    if !rx.is_ready() {
        thread::park_timeout(Duration::from_secs(1));
    }
    let msg = rx.receive();
    assert_eq!(msg, String::from("hello, world!"));
}
