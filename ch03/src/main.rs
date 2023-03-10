//! A basic spin lock with Acquire and Release.

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::time::Duration;

fn main() {
    static mut DATA: String = String::new();
    static LOCKED: AtomicBool = AtomicBool::new(false);
    let verifier = thread::current();
    let work = || loop {
        if !LOCKED.swap(true, Acquire) {
            unsafe {
                DATA.push('!');
            }
            LOCKED.store(false, Release);
            verifier.unpark();
            return;
        }
    };

    thread::scope(|s| {
        // workers.
        (0..100).for_each(|_| {
            s.spawn(work);
        });

        // a verifier.
        loop {
            while LOCKED.load(Acquire) {
                std::hint::spin_loop();
            }
            let len = unsafe { DATA.len() };
            if len == 100 {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }

        unsafe {
            assert_eq!(DATA.len(), 100);
            DATA.chars().for_each(|c| assert_eq!(c, '!'));
        }
    });
}
