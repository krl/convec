use std::cell::UnsafeCell;
use std::ops::Index;
use std::{fmt, mem};

use parking_lot::RwLock;

const BASE: usize = 32;

/// A concurrent vector, only supporting push and indexed access
pub struct ConVec<T> {
    len: RwLock<usize>,
    allocations: [UnsafeCell<Vec<T>>; 64],
}

pub struct ConVecIter<'a, T: 'a> {
    pub(crate) inner: &'a ConVec<T>,
    pub(crate) index: usize,
}

unsafe impl<T> Sync for ConVec<T> {}
unsafe impl<T> Send for ConVec<T> {}

impl<T> ConVec<T> {
    pub fn new() -> Self {
        ConVec {
            len: RwLock::new(0),
            allocations: [
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
                UnsafeCell::new(vec![]),
            ],
        }
    }

    // get the allocation and offset within it.
    fn allocation(&self, mut offset: usize) -> (usize, usize) {
        let mut compare = BASE;
        let mut allocation = 0;

        loop {
            if compare > offset {
                return (allocation, offset);
            } else {
                offset -= compare;
            }
            compare = compare << 1;
            allocation += 1;
        }
    }

    #[inline]
    fn _get(&self, idx: usize) -> &T {
        let (index, offset) = self.allocation(idx);
        unsafe { &(*self.allocations[index].get())[offset] }
    }

    #[inline]
    fn valid_index(&self, idx: usize) -> bool {
        *self.len.read() > idx
    }

    pub fn len(&self) -> usize {
        *self.len.read()
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if self.valid_index(idx) {
            Some(self._get(idx))
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, idx: usize) -> &T {
        self._get(idx)
    }

    pub fn push(&self, t: T) -> usize {
        let mut guard = self.len.write();
        let idx = *guard;
        *guard += 1;
        let (index, _) = self.allocation(idx);
        unsafe {
            let allocation = self.allocations[index].get();
            if (*allocation).len() == 0 {
                *allocation = Vec::with_capacity(BASE << index);
            }
            (*allocation).push(t);
        }
        idx
    }

    pub unsafe fn pop(&self) -> Option<T> {
        let mut guard = self.len.write();
        if *guard == 0 {
            return None;
        }
        *guard -= 1;

        let (index, _) = self.allocation(*guard);
        let popped = (*self.allocations[index].get()).pop();
        // deallocate
        if (*self.allocations[index].get()).len() == 0 {
            *self.allocations[index].get() = vec![];
        }
        popped
    }

    #[allow(unused)]
    pub fn heap_size(&self) -> usize
    where
        T: ::std::fmt::Debug,
    {
        let mut heap_size = 0;
        for i in 0..64 {
            unsafe {
                heap_size += (*self.allocations[i].get()).capacity()
                    * mem::size_of::<T>();
            }
        }
        heap_size
    }
}

impl<T: fmt::Debug> fmt::Debug for ConVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..64 {
            let alloc = unsafe { &(*self.allocations[i].get()) };
            if alloc.len() > 0 {
                write!(f, "{:?} ", alloc)?;
            }
        }
        Ok(())
    }
}

impl<T> Index<usize> for ConVec<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        if self.valid_index(idx) {
            self._get(idx)
        } else {
            panic!("Index out of range");
        }
    }
}

impl<'a, T> Iterator for ConVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.inner.get(self.index);
        if result.is_some() {
            self.index += 1;
        }
        result
    }
}
