//! A safe one-shot borrowed channel.

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Channel<T> {
    ready: AtomicBool,
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
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

    /// Splits the `Channel` into `Sender` and `Receiver`.
    ///
    /// It takes the exclusive reference to avoid the reuse,
    /// as it's a single shot channel.
    pub fn split(&mut self) -> (Sender<T>, Receiver<T>) {
        // drops the previous channel
        *self = Self::new();
        (
            Sender {
                channel: self, // shared ref.
            },
            Receiver {
                channel: self, // shared ref.
            },
        )
    }
}

#[derive(Debug)]
pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Sender<'_, T> {
    pub fn send(self, msg: T) {
        unsafe {
            (*self.channel.data.get()).write(msg);
        }
        self.channel.ready.store(true, Release);
    }
}

#[derive(Debug)]
pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Receiver<'_, T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Relaxed)
    }

    /// # Panics
    ///
    /// It panics when the data is not ready.
    /// check the data readiness by calling
    /// `is_ready()`, instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread;
    ///
    /// let mut channel = Channel::new()
    /// let (tx, rx) = channel.split();
    ///
    /// thread::scope(|s| {
    ///     s.spawn(|| {
    ///         tx.send("Test".to_string());
    ///     });
    ///
    ///     s.spawn(|| {
    ///         while !rx.is_ready() {
    ///             thread::park();
    ///         }
    ///         let msg = rx.receive();
    ///         assert_eq!(msg, "Test".to_string());
    ///     });
    /// });
    /// ```
    pub fn receive(self) -> T {
        if !self.channel.ready.swap(false, Acquire) {
            panic!("message is not ready");
        }
        unsafe { (*self.channel.data.get()).assume_init_read() }
    }
}

fn main() {
    let mut channel = Channel::new();
    let (tx, rx) = channel.split();

    let t = thread::current();
    thread::scope(|s| {
        // a sender.
        s.spawn(|| {
            tx.send("Let's do this!".to_string());
            t.unpark();
        });

        // a receiver.
        while !rx.is_ready() {
            thread::park_timeout(Duration::from_secs(1));
        }
        let msg = rx.receive();
        assert_eq!(msg, String::from("Let's do this!"));
    });
}
