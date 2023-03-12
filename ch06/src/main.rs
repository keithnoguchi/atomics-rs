//! An Arc<T> and Weak<T>

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

#[derive(Debug)]
pub struct Arc<T> {
    weak: Weak<T>,
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.weak.data().data_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe {
                *self.weak.ptr.as_mut().data.get_mut() = None;
            }
        }
    }
}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        Self {
            weak: Weak {
                ptr: NonNull::from(Box::leak(Box::new(Data {
                    data_ref_count: AtomicUsize::new(1),
                    alloc_ref_count: AtomicUsize::new(1),
                    data: UnsafeCell::new(Some(data)),
                }))),
            },
        }
    }
}

#[derive(Debug)]
pub struct Weak<T> {
    ptr: NonNull<Data<T>>,
}

impl<T> Weak<T> {
    fn data(&self) -> &Data<T> {
        unsafe { self.ptr.as_ref() }
    }
}

#[derive(Debug)]
struct Data<T> {
    data_ref_count: AtomicUsize,
    alloc_ref_count: AtomicUsize,
    data: UnsafeCell<Option<T>>,
}

fn main() {
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
    #[derive(Debug)]
    struct DropMonitor;
    impl Drop for DropMonitor {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Relaxed);
        }
    }

    let data = Arc::new((String::from("hello"), DropMonitor));
    println!("{data:?}");
    drop(data);
    assert_eq!(DROP_COUNT.load(Acquire), 1);
}
