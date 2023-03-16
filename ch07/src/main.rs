//! Understanding Processors
//!
//! # Examples
//!
//! ```
//! $ cargo +nightly run --release
//! ```

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Instant;

// A conditional compilation.
//
// https://stackoverflow.com/questions/59542378/conditional-compilation-for-nightly-vs-stable-rust-or-compiler-version
#[cfg(feature = "nightly-features")]
use std::hint::black_box;

#[cfg(not(feature = "nightly-features"))]
#[inline]
const fn black_box<T>(dummy: T) -> T {
    dummy
}

/// 64 byte alignment.
#[repr(align(64))]
struct Aligned(AtomicU64);

static A: [Aligned; 3] = [
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
];

fn main() {
    black_box(&A);

    thread::spawn(|| loop {
        A[0].0.store(0, Relaxed);
    });

    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].0.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}
