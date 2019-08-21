#![deny(missing_docs)]
//! Two views on a concurrent vector

use std::ops::Index;

mod convec;

use convec::{ConVec, ConVecIter};

#[derive(Debug)]
/// Append only concurrent vector
pub struct AoVec<T>(ConVec<T>);
#[derive(Debug)]
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

    #[allow(unused)]
    fn heap_size(&self) -> usize
    where
        T: ::std::fmt::Debug,
    {
        self.0.heap_size()
    }
}

impl<T> Default for ConStack<T> {
    fn default() -> ConStack<T> {
        ConStack::new()
    }
}

impl<'a, T> IntoIterator for &'a ConStack<T> {
    type Item = &'a T;
    type IntoIter = ConVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

impl<T> Default for AoVec<T> {
    fn default() -> AoVec<T> {
        AoVec::new()
    }
}

impl<T> Index<usize> for AoVec<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        self.0.get(idx).expect("Index out of bounds")
    }
}

impl<'a, T> IntoIterator for &'a AoVec<T> {
    type Item = &'a T;
    type IntoIter = ConVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn aovec() {
        let vec = Arc::new(AoVec::new());
        let n = 1_000_000;

        let n_threads = 16;

        let mut handles = vec![];

        for t in 0..n_threads {
            let vec = vec.clone();
            handles.push(std::thread::spawn(move || {
                for i in 0..n {
                    if i % n_threads == t {
                        vec.push(i);
                    }
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
    fn single_threaded_aovec() {
        let vec = AoVec::new();
        let n = 1_000_000;

        for i in 0..n {
            vec.push(i);
        }

        for i in 0..n {
            assert_eq!(vec.get(i), Some(&i));
        }
    }

    #[test]
    fn single_threaded_constack() {
        let stack = ConStack::new();
        let n = 1_000_000;

        for i in 0..n {
            stack.push(i);
        }

        for i in 0..n {
            assert_eq!(stack.pop(), Some(n - i - 1));
        }
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.heap_size(), 0);
    }

    #[test]
    fn constack() {
        let stack = Arc::new(ConStack::new());
        let n = 32;

        let n_threads = 16;

        let mut handles = vec![];

        for t in 0..n_threads {
            let stack = stack.clone();
            handles.push(std::thread::spawn(move || {
                for i in 0..n {
                    if i % n_threads == t {
                        stack.push(i);
                    }
                }
            }))
        }

        for h in handles {
            h.join().unwrap();
        }

        let mut handles = vec![];

        for t in 0..n_threads {
            let stack = stack.clone();
            handles.push(std::thread::spawn(move || {
                for i in 0..n {
                    if i % n_threads == t {
                        stack.pop().is_some();
                    }
                }
            }))
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(stack.heap_size(), 0);
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.pop(), None);
    }
}
