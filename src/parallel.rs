use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{ptr, mem};

// Common x86-64 cache line size
pub const CACHE_LINE_SIZE: usize = 64;

pub struct TrustedThreadSafe<T> {
    inner: UnsafeCell<T>,
}

impl<T> TrustedThreadSafe<T> {
    pub fn new(value: T) -> TrustedThreadSafe<T> {
        TrustedThreadSafe { inner: UnsafeCell::new(value) }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &*self.inner.get() }
    }

    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    pub fn into_inner(self) -> T {
        unsafe { self.inner.into_inner() }
    }
}

unsafe impl<T> Send for TrustedThreadSafe<T> {}

unsafe impl<T> Sync for TrustedThreadSafe<T> {}

pub struct Mapper<T> {
    pub target: TrustedThreadSafe<Vec<T>>,
    pub index: AtomicUsize,
    pub len: usize,
}

impl<T> Mapper<T> {
    pub fn new(len: usize) -> Mapper<T> {
        Mapper {
            target: {
                let mut tmp = Vec::with_capacity(len);
                unsafe { tmp.set_len(len); }
                TrustedThreadSafe::new(tmp)
            },
            index: AtomicUsize::new(0),
            len,
        }
    }

    pub fn into_target(self) -> Vec<T> {
        self.target.into_inner()
    }

    pub fn map<F, U>(&self, data: &[U], mapper: F) where F: Fn(&U) -> T, U: Sync {
        let Mapper { ref target, ref index, len } = *self;

        let fetch_size = CACHE_LINE_SIZE * mem::size_of::<U>();

        let mut_target = target.as_mut();

        loop {
            let mut i = index.fetch_add(fetch_size, Ordering::Relaxed);

            if i < len {
                let max = i + fetch_size;
                let max = if max < len { max } else { len };

                while i < max {
                    unsafe {
                        ptr::write(&mut mut_target[i],
                                   mapper(&data[i]));
                    }

                    i += 1;
                }
            } else {
                break;
            }
        }
    }

    pub fn map_move<F, U>(&self, data: &[U], mapper: F) where F: Fn(U) -> T {
        let Mapper { ref target, ref index, len } = *self;

        let fetch_size = CACHE_LINE_SIZE * mem::size_of::<U>();

        let mut_target = target.as_mut();

        loop {
            let mut i = index.fetch_add(fetch_size, Ordering::Relaxed);

            if i < len {
                let max = i + fetch_size;
                let max = if max < len { max } else { len };

                while i < max {
                    unsafe {
                        ptr::write(&mut mut_target[i],
                                   mapper(ptr::read(&data[i])));
                    }

                    i += 1;
                }
            } else {
                break;
            }
        }
    }
}