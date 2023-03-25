//! Compare-and-block with futex(2)
//!
//! # Examples
//!
//! ```
//! $ uname -o
//! GNU/Linux
//! $ cargo run -q
//! ThreadId(1): state is 0, waiting...
//! ThreadId(2): doing some work
//! ThreadId(2): done. let the executor know
//! ThreadId(1): cool. state is 1
//! ```

#[cfg(not(target_os = "linux"))]
compile_error!("Sorry, Linux only...");

use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

/// Waits, e.g. compare-and-block in the kernel space, if the
/// value `a` *IS* `expected` value.
pub fn wait(a: &AtomicU32, expected: u32) {
    // refer futex(2) for the syscall(2) arguments.
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            a as *const AtomicU32,
            libc::FUTEX_WAIT,
            expected,
        );
    }
}

/// Wakes one thread which is waiting on value `a`.
pub fn wake_one(a: &AtomicU32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            a as *const AtomicU32,
            libc::FUTEX_WAKE,
            1, // just one waiter.
        );
    }
}

fn main() {
    let state = AtomicU32::new(0);

    thread::scope(|s| {
        // A waker.
        s.spawn(|| {
            let id = thread::current().id();
            println!("{id:?}: doing some work");
            thread::sleep(Duration::from_millis(10));
            state.store(1, Relaxed);
            println!("{id:?}: done. let the executor know");
            wake_one(&state);
        });

        // An executor.
        let id = thread::current().id();
        while state.load(Relaxed) == 0 {
            // atomically compare-and-block if the state is 0.
            println!("{id:?}: state is {}, waiting...", state.load(Relaxed));
            wait(&state, 0);
        }
        println!("{id:?}: cool. state is {}", state.load(Relaxed));
    });
}
