//! Demonstration of the memory ordering bug.
//!
//! # Examples
//!
//! ```
//! $ for i in {0..100}; do cargo run -rq; done
//! ```
//!
//! Please run it on ARM64 to reveal the bug.

use std::sync::atomic::compiler_fence;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::thread;

fn main() {
    let lock = AtomicBool::new(false);
    let counter = AtomicUsize::new(0);

    thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..1_000_000 {
                // Acquires the lock, using the wrong memory ordering.
                while lock.swap(true, Relaxed) {}
                compiler_fence(Acquire);

                // Non-atomically increments the counter,
                // while holding the lock.
                let old = counter.load(Relaxed);
                let new = old + 1;
                counter.store(new, Relaxed);

                // Releases the lock, using the wrong memory ordering.
                compiler_fence(Release);
                lock.store(false, Relaxed);
            }
        });
    });

    println!("{}", counter.into_inner());
}
