//! A memory fence with Acquire, Relaxed, and Release

use std::array;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::time::Duration;

static mut DATA: [u32; 10] = [0; 10];

#[allow(clippy::declare_interior_mutable_const)]
const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];

fn main() {
    let work = |id: usize| {
        let id = id as u32;
        thread::sleep(Duration::from_millis(id as u64 % 3 + 1));
        id
    };
    // workers.
    for i in 0..10 {
        thread::spawn(move || {
            let data = work(i);
            unsafe {
                DATA[i] = data;
            }
            READY[i].store(true, Release);
        });
    }

    thread::sleep(Duration::from_millis(2));

    // a repoter.
    let ready: [bool; 10] = array::from_fn(|i| READY[i].load(Relaxed));
    if ready.contains(&true) {
        fence(Acquire);
        for i in 0..10 {
            if ready[i] {
                let data = unsafe { DATA[i] };
                println!("DATA[{i}] = {data}");
            }
        }
    }
}
