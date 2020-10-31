//! The Intern Pool  

use std::{
    borrow::Borrow,
    ffi::OsStr,
    hash::Hash,
    ops::Deref,
    sync::{Arc, RwLock},
};

use dashmap::DashSet;
use once_cell::sync::Lazy;

/// The String Intern Pool  
pub static STR_POOL: Lazy<Pool<str>> = Lazy::new(|| Pool::new());

/// The OsString Intern Pool  
pub static OS_STR_POOL: Lazy<Pool<OsStr>> = Lazy::new(|| Pool::new());

/// The Intern Pool  
#[derive(Debug)]
pub struct Pool<T: Eq + Hash + ?Sized> {
    pool: DashSet<Arc<T>>,
    gc_lock: RwLock<()>,
}

impl<T: Eq + Hash + ?Sized> Pool<T> {
    /// New a empty intern pool
    #[inline]
    pub fn new() -> Self {
        Self {
            pool: DashSet::new(),
            gc_lock: RwLock::new(()),
        }
    }
}

impl<T: Eq + Hash + ?Sized> Pool<T> {
    /// Make a intern
    #[inline]
    pub fn intern<A: AsRef<T>>(&self, a: A, to_arc: impl FnOnce(A) -> Arc<T>) -> Intern<T> {
        match self.pool.get(a.as_ref()).map(|v| v.key().clone()) {
            Some(v) => Intern(v),
            None => {
                let arc = to_arc(a);
                Intern(self.insert_arc(arc))
            }
        }
    }

    #[inline]
    fn insert_arc(&self, arc: Arc<T>) -> Arc<T> {
        if self.pool.insert(Clone::clone(&arc)) {
            arc
        } else {
            self.when_failed(arc)
        }
    }

    #[cold]
    fn when_failed(&self, arc: Arc<T>) -> Arc<T> {
        let lock = self.gc_lock.read();
        let r = match self.pool.get(arc.as_ref()).map(|v| v.key().clone()) {
            Some(v) => v,
            None => {
                let s = self.pool.insert(Clone::clone(&arc));
                assert!(s);
                arc
            }
        };
        drop(lock);
        r
    }
}

impl<T: Eq + Hash + ?Sized> Pool<T> {
    /// Delete all interning string with reference count == 1 in the pool
    pub fn collect_garbage(&self) {
        let lock = self.gc_lock.write();
        self.pool.retain(|arc| Arc::<T>::strong_count(arc) > 1);
        drop(lock);
    }
}

/// Intern Ptr  
#[derive(Debug, Eq, Ord, PartialOrd)]
pub struct Intern<T: ?Sized>(Arc<T>);

impl<T: ?Sized> Intern<T> {
    /// Get target ref
    #[inline]
    pub fn get(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: ?Sized> PartialEq for Intern<T> {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::<T>::as_ptr(&self.0) == std::sync::Arc::<T>::as_ptr(&other.0)
    }
}

impl<T: ?Sized> Clone for Intern<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> Deref for Intern<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: ?Sized> AsRef<T> for Intern<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: ?Sized> Borrow<T> for Intern<T> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T: ?Sized> From<Intern<T>> for Arc<T> {
    fn from(v: Intern<T>) -> Self {
        v.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let h = STR_POOL.intern("asd", Arc::from);
        assert_eq!(h.get(), "asd");
    }

    #[test]
    fn test_same() {
        let h1 = STR_POOL.intern("asd", Arc::from);
        let h2 = STR_POOL.intern("asd", Arc::from);
        assert_eq!(h1, h2);
        assert_eq!(h1.get(), "asd");
        assert_eq!(h2.get(), "asd");
    }

    #[test]
    fn test_not_same() {
        let h1 = STR_POOL.intern("asd", Arc::from);
        let h2 = STR_POOL.intern("123", Arc::from);
        assert_ne!(h1, h2);
        assert_eq!(h1.get(), "asd");
        assert_eq!(h2.get(), "123");
    }

    #[test]
    #[ignore]
    fn test_pool_gc() {
        assert_eq!(STR_POOL.pool.len(), 0);
        STR_POOL.intern("asd", Arc::from);
        assert_eq!(STR_POOL.pool.len(), 1);
        let h = STR_POOL.intern("123", Arc::from);
        assert_eq!(STR_POOL.pool.len(), 2);
        STR_POOL.collect_garbage();
        assert_eq!(STR_POOL.pool.len(), 1);
        drop(h);
        assert_eq!(STR_POOL.pool.len(), 1);
        STR_POOL.collect_garbage();
        assert_eq!(STR_POOL.pool.len(), 0);
    }

    #[test]
    fn test_concurrent_1() {
        use std::thread::spawn;

        let t: Vec<_> = (0..100)
            .into_iter()
            .map(|i| {
                spawn(move || {
                    let a = STR_POOL.intern(i.to_string(), Arc::from);
                    let v: Vec<_> = (0..100)
                        .into_iter()
                        .map(|_| spawn(move || STR_POOL.intern(i.to_string(), Arc::from)))
                        .collect();
                    for b in v.into_iter() {
                        assert_eq!(a, b.join().unwrap());
                    }
                })
            })
            .collect();

        for r in t.into_iter() {
            assert!(r.join().is_ok());
        }
    }

    #[test]
    fn test_concurrent_2_gc() {
        use std::thread::spawn;

        let t: Vec<_> = (0..100)
            .into_iter()
            .map(|i| {
                spawn(move || {
                    let v: Vec<_> = (0..100)
                        .into_iter()
                        .map(|_| spawn(move || STR_POOL.intern(i.to_string(), Arc::from)))
                        .collect();
                    for b in v.into_iter() {
                        assert_eq!(b.join().unwrap().get(), i.to_string());
                    }
                })
            })
            .zip((0..100).into_iter().map(|_| {
                spawn(move || {
                    let v: Vec<_> = (0..100)
                        .into_iter()
                        .map(|_| {
                            spawn(move || {
                                STR_POOL.collect_garbage();
                            })
                        })
                        .collect();
                    for r in v.into_iter() {
                        assert!(r.join().is_ok());
                    }
                })
            }))
            .collect();

        for (a, b) in t.into_iter() {
            assert!(a.join().is_ok());
            assert!(b.join().is_ok());
        }
    }
}
