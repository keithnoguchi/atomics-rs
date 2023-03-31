//! A Mutex<T> with two states
//!
//! # Examples
//!
//! ```
//! $ cargo +nightly run -rq
//! ```

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Instant;

#[derive(Debug)]
pub struct Mutex<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }
}

fn main() {
    let m = Mutex::new(0);
    #[cfg(feature = "nightly-features")]
    std::hint::black_box(&m);

    let start = Instant::now();
    thread::scope(|s| {
    });
    let duration = start.elapsed();

    println!("{:?}", duration);
}
