//! An Arc<T> and Weak<T>

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::sync::atomic::AtomicUsize;

#[derive(Debug)]
struct Data<T> {
    _data_ref_count: AtomicUsize,
    _alloc_ref_count: AtomicUsize,
    _data: UnsafeCell<Option<T>>,
}

impl<T> Default for Data<T> {
    fn default() -> Self {
        Self {
            _data_ref_count: AtomicUsize::new(0),
            _alloc_ref_count: AtomicUsize::new(0),
            _data: UnsafeCell::new(None),
        }
    }
}

fn main() {
    let data = Data::<String>::default();

    println!("{data:?}");
}
