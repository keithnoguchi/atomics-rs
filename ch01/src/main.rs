use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let q = Mutex::new(VecDeque::new());
    let ready = Condvar::new();

    thread::scope(|s| {
        // consumers.
        for _ in 0..100 {
            s.spawn(|| loop {
                let mut q = q.lock().unwrap();
                let item = loop {
                    if let Some(item) = q.pop_front() {
                        break item;
                    } else {
                        q = ready.wait(q).unwrap();
                    }
                };
                drop(q);
                dbg!(item);
            });
        }

        // a producer.
        for i in 0.. {
            q.lock().unwrap().push_back(i);
            ready.notify_one();
            thread::sleep(Duration::from_secs(1));
        }
    });
}
