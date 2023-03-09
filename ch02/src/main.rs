//! Multi worker job reporting with fetch_add().

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

fn main() {
    let completed = &AtomicUsize::new(0);
    let total_jobs = || {
        static NR_TASKS: AtomicUsize = AtomicUsize::new(0);
        loop {
            let nr_tasks = NR_TASKS.load(Relaxed);
            if nr_tasks != 0 {
                break nr_tasks;
            } else {
                NR_TASKS.store(4 * 30, Relaxed);
                dbg!(NR_TASKS.load(Relaxed));
            }
        }
    };
    let main_thread = thread::current();
    let work = |id| {
        thread::sleep(Duration::from_millis(100));
        if id % 6 == 0 {
            main_thread.unpark();
        }
    };

    thread::scope(|s| {
        // workers.
        (0..4).for_each(|id| {
            s.spawn(move || {
                let jobs = total_jobs() / 4;
                (0..jobs).for_each(|item_id| {
                    work(jobs * id + item_id);
                    completed.fetch_add(1, Relaxed);
                });
            });
        });

        // a reporter.
        loop {
            let completed = completed.load(Relaxed);
            println!("{completed}/{} done", total_jobs());
            if completed == total_jobs() {
                break;
            }
            thread::park_timeout(Duration::from_secs(1));
        }
    });
}
