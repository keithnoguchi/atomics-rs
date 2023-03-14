//! Understanding Processors

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

#[cfg(feature = "nightly-features")]
use std::hint::black_box;

#[cfg(not(feature = "nightly-features"))]
pub const fn black_box<T>(dummy: T) -> T {
    dummy
}

static A: AtomicU64 = AtomicU64::new(0);

fn main() {
    let start = Instant::now();
    black_box(&A);
    for _ in 0..1_000_000_000 {
        black_box(A.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}
