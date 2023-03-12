//! An Arc<T>

#![forbid(missing_debug_implementations)]

use std::ptr::NonNull;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed};

#[derive(Debug)]
pub struct Arc<T> {
    ptr: NonNull<Data<T>>,
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Relaxed) == 1 {
            fence(Acquire);
            drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
        }
    }
}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        Self {
            ptr: NonNull::from(Box::leak(Box::new(Data {
                ref_count: AtomicUsize::new(1),
                _data: data,
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
    _data: T,
}

fn main() {
    static DROPPED: AtomicUsize = AtomicUsize::new(0);
    #[derive(Debug)]
    struct DropMonitor;
    impl Drop for DropMonitor {
        fn drop(&mut self) {
            DROPPED.fetch_add(1, Relaxed);
        }
    }

    let arc = Arc::new(("hello", DropMonitor));
    println!("{arc:?}");
    drop(arc);
    assert_eq!(DROPPED.load(Relaxed), 1);
}
