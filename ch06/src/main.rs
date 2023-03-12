//! An Arc<T>

#![forbid(missing_debug_implementations)]

use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed};
use std::thread;

#[derive(Debug)]
pub struct Arc<T> {
    ptr: NonNull<Data<T>>,
}

unsafe impl<T> Send for Arc<T> where T: Send + Sync {}
unsafe impl<T> Sync for Arc<T> where T: Send + Sync {}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        self.data().ref_count.fetch_add(1, Relaxed);
        Self { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Relaxed) == 1 {
            fence(Acquire);
            drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data().data
    }
}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        Self {
            ptr: NonNull::from(Box::leak(Box::new(Data {
                ref_count: AtomicUsize::new(1),
                data,
            }))),
        }
    }

    fn data(&self) -> &Data<T> {
        unsafe { self.ptr.as_ref() }
    }
}

#[derive(Debug)]
struct Data<T> {
    ref_count: AtomicUsize,
    data: T,
}

fn main() {
    #[derive(Debug)]
    struct DropMonitor;
    static DROPPED: AtomicUsize = AtomicUsize::new(0);
    impl Drop for DropMonitor {
        fn drop(&mut self) {
            DROPPED.fetch_add(1, Relaxed);
        }
    }

    let data = Arc::new(("hello".to_string(), DropMonitor));
    println!("{:?}: {data:?}, {:?}", thread::current().id(), *data);
    thread::scope(|s| {
        // workers.
        for _ in 0..5 {
            let data = data.clone();
            s.spawn(move || {
                for _ in 0..1000 {
                    let _data = data.clone();
                }
                println!("{:?}: {data:?}, {:?}", thread::current().id(), *data);
            });
        }
    });
    drop(data);
    assert_eq!(DROPPED.load(Relaxed), 1);
}
