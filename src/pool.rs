//! The String Intern Pool  

use std::sync::{Arc, RwLock};

use dashmap::DashSet;
use once_cell::sync::Lazy;

static POOL: Lazy<DashSet<Arc<str>>> = Lazy::new(|| DashSet::new());
static GC_LOCK: Lazy<RwLock<()>> = Lazy::new(|| RwLock::new(()));

#[derive(Debug, Clone, Eq, Ord, PartialOrd)]
pub(crate) struct Handle(Arc<str>);

impl Handle {
    #[inline]
    pub fn new<S: AsRef<str>>(s: S, to_arc: impl FnOnce(S) -> Arc<str>) -> Self {
        match POOL.get(s.as_ref()).map(|v| v.key().clone()) {
            Some(v) => Self(v),
            None => {
                let arc = to_arc(s);
                Self(insert_arc(arc))
            }
        }
    }

    #[inline]
    pub fn get(&self) -> &str {
        self.0.as_ref()
    }
}

#[inline]
fn insert_arc(arc: Arc<str>) -> Arc<str> {
    if POOL.insert(Clone::clone(&arc)) {
        arc
    } else {
        #[cold]
        fn when_failed(arc: Arc<str>) -> Arc<str> {
            let lock = GC_LOCK.read();
            let r = match POOL.get(arc.as_ref()).map(|v| v.key().clone()) {
                Some(v) => v,
                None => {
                    let s = POOL.insert(Clone::clone(&arc));
                    assert!(s);
                    arc
                }
            };
            drop(lock);
            r
        }
        when_failed(arc)
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

/// Delete all interning string with reference count == 1 in the pool
pub fn collect_garbage() {
    let lock = GC_LOCK.write();
    POOL.retain(|arc| Arc::<str>::strong_count(arc) > 1);
    drop(lock);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let h = Handle::new("asd", Arc::from);
        assert_eq!(h.get(), "asd");
    }

    #[test]
    fn test_same() {
        let h1 = Handle::new("asd", Arc::from);
        let h2 = Handle::new("asd", Arc::from);
        assert_eq!(h1, h2);
        assert_eq!(h1.get(), "asd");
        assert_eq!(h2.get(), "asd");
    }

    #[test]
    fn test_not_same() {
        let h1 = Handle::new("asd", Arc::from);
        let h2 = Handle::new("123", Arc::from);
        assert_ne!(h1, h2);
        assert_eq!(h1.get(), "asd");
        assert_eq!(h2.get(), "123");
    }

    #[test]
    #[ignore]
    fn test_pool_gc() {
        assert_eq!(POOL.len(), 0);
        Handle::new("asd", Arc::from);
        assert_eq!(POOL.len(), 1);
        let h = Handle::new("123", Arc::from);
        assert_eq!(POOL.len(), 2);
        collect_garbage();
        assert_eq!(POOL.len(), 1);
        drop(h);
        assert_eq!(POOL.len(), 1);
        collect_garbage();
        assert_eq!(POOL.len(), 0);
    }

    #[test]
    fn test_concurrent_1() {
        use std::thread::spawn;

        let t: Vec<_> = (0..100)
            .into_iter()
            .map(|i| {
                spawn(move || {
                    let a = Handle::new(i.to_string(), Arc::from);
                    let v: Vec<_> = (0..100)
                        .into_iter()
                        .map(|_| spawn(move || Handle::new(i.to_string(), Arc::from)))
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
                        .map(|_| spawn(move || Handle::new(i.to_string(), Arc::from)))
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
                                collect_garbage();
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
