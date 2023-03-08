use std::io::stdin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    static STOP: AtomicBool = AtomicBool::new(false);
    let work = || thread::sleep(Duration::from_millis(10));

    // A worker thread.
    let worker = thread::spawn(move || {
        while !STOP.load(Relaxed) {
            work();
        }
    });

    // A main thread supporting the operator.
    for line in stdin().lines() {
        match line?.as_str() {
            "q" | "quit" => break,
            "help" => println!("usage: q"),
            word => println!("got {word:?}, type \"q\"."),
        }
    }

    // Set the stop flag and wait for the worker to complete.
    STOP.store(true, Relaxed);
    worker.join().expect("worker paniced");

    Ok(())
}
