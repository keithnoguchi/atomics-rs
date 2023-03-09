//! Counter with compare_exchange, instead of fetch_add

use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

fn main() {
    let counter = &AtomicU32::new(0);
    let increment = || {
        let mut current = counter.load(Relaxed);
        loop {
            let new = current + 1;
            match counter.compare_exchange(current, new, Relaxed, Relaxed) {
                Ok(_) => return,
                Err(v) => current = v,
            }
        }
    };
    let work = |id| {
        let delay = 100 * id as u64;
        thread::sleep(Duration::from_millis(delay));
        increment();
    };

    thread::scope(|s| {
        // workers.
        (0..10).for_each(|id| {
            s.spawn(move || {
                (0..20).for_each(|_| work(id));
                println!("worker{id} done");
            });
        });

        // a reporter.
        loop {
            let count = counter.load(Relaxed);
            println!("current count is {count}");
            if count == 200 {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}
