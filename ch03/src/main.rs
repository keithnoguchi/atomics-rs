//! A lazy one-time non-atomic value initialization

use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(PartialEq)]
struct Data;

fn main() {
    let get_data = || -> &'static Data {
        static PTR: AtomicPtr<Data> = AtomicPtr::new(ptr::null_mut());

        let mut p = PTR.load(Acquire);
        if p.is_null() {
            let data = Box::into_raw(Box::new(Data));
            if let Err(v) = PTR.compare_exchange(ptr::null_mut(), data, Release, Acquire) {
                // lose the race.
                drop(unsafe { Box::from_raw(p) });
                p = v;
            }
        }
        unsafe { &*p }
    };

    let result = Arc::new(Mutex::new(HashMap::new()));
    thread::scope(|s| {
        // get the data from 100 workers.
        (0..100).for_each(|id| {
            let result = result.clone();
            s.spawn(move || {
                result.lock().unwrap().insert(id, get_data());
            });
        });

        // wait for the completion.
        loop {
            let result = result.lock().unwrap().len();
            if result == 100 {
                break;
            }
            println!("waiting...");
            thread::park_timeout(Duration::from_secs(1));
        }
        let expected = get_data();
        let correct = result
            .lock()
            .unwrap()
            .values()
            .filter(|&got| *got == expected)
            .count();
        assert_eq!(correct, 100);
    });
}
