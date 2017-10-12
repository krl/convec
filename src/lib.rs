#![deny(missing_docs)]
//! Two views on a concurrent vector
extern crate parking_lot;

use std::ops::Index;

mod convec;

use convec::ConVec;

/// Append only concurrent vector
pub struct AoVec<T>(ConVec<T>);
/// Concurrent stack
pub struct ConStack<T>(ConVec<T>);

impl<T> ConStack<T> {
    /// Creates a new `ConStack`
    pub fn new() -> Self {
        ConStack(ConVec::new())
    }
    /// Returns the length of the `ConStack`
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Push an element to the `ConStack`
    pub fn push(&self, t: T) {
        self.0.push(t);
    }
    /// Pops the last element off the list (if any)
    pub fn pop(&self) -> Option<T> {
        unsafe { self.0.pop() }
    }
}

impl<T> AoVec<T> {
    /// Creates a new `AoVece`
    pub fn new() -> Self {
        AoVec(ConVec::new())
    }
    /// Returns the length of the `ConStack`.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Push an element to the `AoVec`, returning its index
    pub fn push(&self, t: T) -> usize {
        self.0.push(t)
    }
    /// Get value at index `idx`
    pub fn get(&self, i: usize) -> Option<&T> {
        self.0.get(i)
    }
    /// Get value at index `idx`, without checking bounds
    pub unsafe fn get_unchecked(&self, i: usize) -> &T {
        self.0.get_unchecked(i)
    }
}

impl<T> Index<usize> for AoVec<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        self.0.get(idx).expect("Index out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::collections::HashSet;

    #[test]
    fn aovec() {
        let vec = Arc::new(AoVec::new());
        let n = 1_000_000;

        let n_threads = 16;

        let mut handles = vec![];

        for t in 0..n_threads {
            let vec = vec.clone();
            handles.push(std::thread::spawn(move || for i in 0..n {
                if i % n_threads == t {
                    vec.push(i);
                }
            }))
        }

        for h in handles {
            h.join().unwrap();
        }

        let mut set_index = HashSet::new();
        let mut set_get = HashSet::new();
        let mut set_get_unchecked = HashSet::new();

        for i in 0..n {
            set_index.insert(vec[i]);
            set_get.insert(vec.get(i));
            set_get_unchecked.insert(unsafe { vec.get_unchecked(i) });
        }

        assert_eq!(set_index.len(), n);
        assert_eq!(set_get.len(), n);
        assert_eq!(set_get_unchecked.len(), n);
    }

    #[test]
    fn constack() {
        let vec = Arc::new(ConStack::new());
        let n = 1_000_000;

        let n_threads = 16;

        let mut handles = vec![];

        for t in 0..n_threads {
            let vec = vec.clone();
            handles.push(std::thread::spawn(move || for i in 0..n {
                if i % n_threads == t {
                    vec.push(i);
                }
            }))
        }

        for h in handles {
            h.join().unwrap();
        }

        let mut handles = vec![];

        for t in 0..n_threads {
            let vec = vec.clone();
            handles.push(std::thread::spawn(move || for i in 0..n {
                if i % n_threads == t {
                    vec.pop().is_some();
                }
            }))
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(vec.len(), 0);
        assert_eq!(vec.pop(), None);
    }
}
