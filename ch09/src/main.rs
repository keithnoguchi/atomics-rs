//! A mutex
//!
//! # Examples
//!
//! ```
//! ```

#![forbid(unsafe_code, missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::sync::atomic::AtomicU32;

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

    dbg!(m);
}
