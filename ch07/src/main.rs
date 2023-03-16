//! Understanding Processors

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

static A: [AtomicU64; 3] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

fn main() {
    black_box(&A);

    thread::spawn(|| loop {
        A[0].store(0, Relaxed);
        A[2].store(0, Relaxed);
    });

    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}
