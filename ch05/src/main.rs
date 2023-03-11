//! A channel
#![forbid(missing_debug_implementations)]

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::thread;

#[derive(Debug)]
pub struct Channel<T> {
    q: Mutex<VecDeque<T>>,
    is_ready: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            q: Mutex::new(VecDeque::new()),
            is_ready: Condvar::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.q.lock().unwrap().is_empty()
    }

    pub fn send(&self, msg: T) {
        self.q.lock().unwrap().push_back(msg);
        self.is_ready.notify_one();
    }

    pub fn recv(&self) -> T {
        let mut q = self.q.lock().unwrap();
        loop {
            if let Some(msg) = q.pop_front() {
                return msg;
            }
            q = self.is_ready.wait(q).unwrap();
        }
    }
}

fn main() {
    let ch = &Channel::new();

    thread::scope(|s| {
        // producers.
        for id in 0..5 {
            s.spawn(move || {
                for msg in 0..1000 {
                    ch.send(format!("worker{id}: #{msg} msg"));
                }
            });
        }

        // consumers.
        for _ in 0..10 {
            s.spawn(|| {
                for _ in 0..500 {
                    let _ = ch.recv();
                }
            });
        }
    });

    assert!(ch.is_empty());
}
