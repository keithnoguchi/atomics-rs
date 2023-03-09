//! Statistics with `fetch_add()` and `fetch_max()`

use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    const WORKERS: usize = 4;
    const JOBS: usize = 100;
    let processed = AtomicUsize::new(0);
    let total_time = AtomicU64::new(0);
    let max_time = AtomicU64::new(0);

    thread::scope(|s| {
        // workers.
        let work = &|_id| thread::sleep(Duration::from_millis(98));
        s.spawn(|| {
            (0..WORKERS).for_each(|id| {
                (0..(JOBS / WORKERS)).for_each(|item_id| {
                    let start = Instant::now();
                    work((JOBS / WORKERS) * id + item_id);
                    let duration = start.elapsed().as_micros() as u64;
                    processed.fetch_add(1, Relaxed);
                    total_time.fetch_add(duration, Relaxed);
                    max_time.fetch_max(duration, Relaxed);
                });
            });
        });

        // A reporter.
        let report = |processed| {
            let duration = Duration::from_micros(total_time.load(Relaxed));
            let peak = Duration::from_micros(max_time.load(Relaxed));
            println!(
                "{}/{} processed, {:?} process time, {:?} peak duration.",
                processed, JOBS, duration, peak,
            );
        };
        loop {
            let processed = processed.load(Relaxed);
            if processed == 0 {
                thread::sleep(Duration::from_secs(1));
                continue;
            } else if processed == JOBS {
                report(processed);
                break;
            } else {
                report(processed);
                thread::sleep(Duration::from_secs(1));
            }
        }
    });
}
