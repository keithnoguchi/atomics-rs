//! A channel with `Mutex` and `Condvar`.

#![forbid(unsafe_code, missing_debug_implementations)]

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let ch = Mutex::new(VecDeque::new());
    let ready = Condvar::new();

    thread::scope(|s| {
        // consumers.
        for _ in 0..100 {
            s.spawn(|| loop {
                let mut ch = ch.lock().unwrap();
                let item = loop {
                    if let Some(item) = ch.pop_front() {
                        break item;
                    } else {
                        ch = ready.wait(ch).unwrap();
                    }
                };
                drop(ch);
                dbg!(item);
            });
        }

        // a producer.
        for i in 0.. {
            ch.lock().unwrap().push_back(i);
            ready.notify_one();
            thread::sleep(Duration::from_secs(1));
        }
    });
}
