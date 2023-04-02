//! A Mutex<T> with spins
//!
//! # Examples
//!
//! ```
//! $ cargo +nightly run -rq
//! ```

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::sync::atomic::AtomicU32;

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
    let m = Mutex::new(0);
    #[cfg(features = "nightly-features")]
    std::hint::black_box(&m);

    dbg!(m);
}
