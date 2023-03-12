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

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        let weak = self.weak.clone();
        if self.weak.data().data_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Self { weak }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.weak.data().data_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            let ptr = self.weak.data().data.get();
            unsafe {
                *ptr = None;
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

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().alloc_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Self { ptr: self.ptr }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
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
    let arc = Arc::new(String::from("hello"));
    dbg!(arc);
}