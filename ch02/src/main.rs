//! ID allocation without the overflow

use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

fn main() {
    static NEXT_ID: AtomicU8 = AtomicU8::new(0);
    let get_id = || {
        let mut id = NEXT_ID.load(Relaxed);
        loop {
            assert!(id < 5, "too many IDs");
            match NEXT_ID.compare_exchange(id, id + 1, Relaxed, Relaxed) {
                Ok(_) => break id,
                Err(v) => id = v,
            }
        }
    };

    thread::scope(|s| {
        // workers.
        (0..6).for_each(|id| {
            s.spawn(move || {
                let tid = thread::current().id();
                let delay = id % 3_u64 * 100;
                thread::sleep(Duration::from_millis(delay));
                let retrieved_id = get_id();
                println!("worker{id} {tid:?} with {retrieved_id} done");
            });
        });

        // a reporter.
        loop {
            let id = NEXT_ID.load(Relaxed);
            if id == 5 {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}
