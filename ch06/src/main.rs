//! An Arc<T> and Weak<T>

#![forbid(missing_debug_implementations)]

use std::cell::UnsafeCell;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;

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

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self.weak.data().data.get();
        // Safety: There is a data as it's Arc.
        unsafe { (*ptr).as_ref().unwrap() }
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

    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if this.weak.data().alloc_ref_count.load(Relaxed) == 1 {
            fence(Acquire);
            // Safety: nothing else can access the data.
            let arcdata = unsafe { this.weak.ptr.as_mut() };
            let option = arcdata.data.get_mut();
            // Safety: the data is still there based on the
            // alloc_ref_count above.
            let data = option.as_mut().unwrap();
            Some(data)
        } else {
            None
        }
    }

    pub fn downgrade(this: &Self) -> Weak<T> {
        this.weak.clone()
    }
}

#[derive(Debug)]
pub struct Weak<T> {
    ptr: NonNull<Data<T>>,
}

unsafe impl<T> Send for Weak<T> where T: Send + Sync {}
unsafe impl<T> Sync for Weak<T> where T: Send + Sync {}

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
    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut n = self.data().data_ref_count.load(Relaxed);
        loop {
            if n == 0 {
                // there is no data.
                return None;
            }
            assert!(n < usize::MAX);
            match self
                .data()
                .data_ref_count
                .compare_exchange(n, n + 1, Acquire, Relaxed)
            {
                Ok(_) => return Some(Arc { weak: self.clone() }),
                Err(v) => n = v,
            }
        }
    }

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
    let mut data = Arc::new((String::from("hello"), DropMonitor));

    // test Arc::get_mut().
    let (string, _) = Arc::get_mut(&mut data).unwrap();
    string.push_str(", world!");

    thread::scope(|s| {
        // tests Send for Arc<T>.
        for _ in 0..10 {
            let data = data.clone();
            s.spawn(move || {
                dbg!(data);
                assert_eq!(DROP_COUNT.load(Relaxed), 0);
            });
        }

        // tests Sync for Arc<T>.
        for _ in 0..5 {
            s.spawn(|| {
                eprintln!("{:?}: {data:#?}", thread::current());
                assert_eq!(DROP_COUNT.load(Relaxed), 0);
            });
        }

        // test Arc::<T>::downgrade().
        for _ in 0..5 {
            let weak = Arc::downgrade(&data);
            s.spawn(|| {
                dbg!(weak);
                assert_eq!(DROP_COUNT.load(Relaxed), 0);
            });
        }
    });
    assert_eq!(DROP_COUNT.load(Relaxed), 0);
    dbg!(&*data);
    let weak = Arc::downgrade(&data);
    let upgraded = weak.upgrade();
    dbg!(upgraded);
    dbg!(data);
    assert_eq!(DROP_COUNT.load(Relaxed), 1);
    // make sure it's still there, even if the data itself
    // had been dropped.
    dbg!(weak);
}
