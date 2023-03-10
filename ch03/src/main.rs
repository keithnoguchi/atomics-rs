//! Acquire and Release memory ordering

use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Duration;

fn main() {
    static DATA: AtomicI32 = AtomicI32::new(0);
    static READY: AtomicBool = AtomicBool::new(false);

    let main_thread = thread::current();
    thread::spawn(move || {
        DATA.store(123, Relaxed);
        thread::sleep(Duration::from_millis(500));
        READY.store(true, Release); // everything before `Release`
                                    // happens before across threads.
        main_thread.unpark();
    });

    while !READY.load(Acquire) {
        // everything after `Acquire` happens after across threads.
        println!("waiting...");
        thread::park_timeout(Duration::from_secs(1));
    }
    assert_eq!(DATA.load(Relaxed), 123);
}
