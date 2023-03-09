//! Lazy One-time Initialization with Race

use std::collections::HashMap;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let start = Instant::now();
    let get_key = || {
        static KEY: AtomicI8 = AtomicI8::new(-1);
        let key = KEY.load(Relaxed);
        if key == -1 {
            let new_key = start.elapsed().as_nanos() as i8;
            match KEY.compare_exchange(-1, new_key, Relaxed, Relaxed) {
                Ok(_) => new_key,
                Err(key) => key,
            }
        } else {
            key
        }
    };

    let reporter = &thread::current();
    let delay = &|id| Duration::from_millis(id % 3_u64 + 1) * 100;
    let result = &Arc::new(Mutex::new(HashMap::new()));
    thread::scope(|s| {
        // let the workers to race, like miners.
        (0..100).for_each(|id| {
            s.spawn(move || {
                thread::sleep(delay(id));
                let key = get_key();
                result.lock().unwrap().insert(id, key);
                reporter.unpark();
            });
        });

        // make sure everyone has the same key.
        let done = || result.lock().unwrap().len() == 100;
        loop {
            if !done() {
                thread::park_timeout(Duration::from_secs(1));
            } else {
                let expected = get_key();
                let wrong = result
                    .lock()
                    .unwrap()
                    .values()
                    .filter(|&got| got == &expected)
                    .count();
                assert!(wrong == 100, "some values are wrong");
                break;
            }
        }
    });
}
