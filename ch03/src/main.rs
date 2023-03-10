//! Acquire and Release memory ordering (unsafe)

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::time::Duration;

fn main() {
    static READY: AtomicBool = AtomicBool::new(false);
    static mut DATA: i32 = 0;

    let main_thread = thread::current();
    thread::spawn(move || {
        unsafe {
            DATA = 123;
        }
        READY.store(true, Release);
        main_thread.unpark();
    });

    while !READY.load(Acquire) {
        println!("waiting worker...");
        thread::park_timeout(Duration::from_secs(1));
    }
    assert_eq!(unsafe { DATA }, 123);
}
