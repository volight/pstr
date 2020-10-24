//! The String Intern Pool  

use std::sync::RwLock;

use dashmap::DashSet;
use once_cell::sync::Lazy;

use crate::prc::Prc;

static POOL: Lazy<DashSet<Prc<str>>> = Lazy::new(|| DashSet::new());
static GC_LOCK: Lazy<RwLock<()>> = Lazy::new(|| RwLock::new(()));

#[derive(Debug, Clone, Eq, Ord, PartialOrd)]
pub(crate) struct Handle(Prc<str>);

impl Handle {
    #[inline]
    pub fn new(s: &str) -> Self {
        match POOL.get(s).map(|v| v.key().clone()) {
            Some(v) => Self(v),
            None => {
                let prc = Prc::from_box(Box::from(s));
                Self(insert_prc(prc))
            }
        }
    }
    #[inline]
    pub fn from_box(s: Box<str>) -> Self {
        match POOL.get(s.as_ref()).map(|v| v.key().clone()) {
            Some(v) => Self(v),
            None => {
                let prc = Prc::from_box(s);
                Self(insert_prc(prc))
            }
        }
    }

    #[inline]
    pub fn get(&self) -> &str {
        self.0.as_ref()
    }
}

#[inline]
fn insert_prc(prc: Prc<str>) -> Prc<str> {
    if POOL.insert(Clone::clone(&prc)) {
        prc
    } else {
        #[cold]
        fn when_failed(prc: Prc<str>) -> Prc<str> {
            let lock = GC_LOCK.read();
            let r = match POOL.get(prc.as_ref()).map(|v| v.key().clone()) {
                Some(v) => v,
                None => {
                    let s = POOL.insert(Clone::clone(&prc));
                    assert!(s);
                    prc
                }
            };
            drop(lock);
            r
        }
        when_failed(prc)
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        self.0.inner_ptr_usize() == other.0.inner_ptr_usize()
    }
}

/// Delete all interning string with reference count == 1 in the pool
pub fn collect_garbage() {
    let lock = GC_LOCK.write();
    POOL.retain(|prc| Prc::<str>::strong_count(prc) > 1);
    drop(lock);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let h = Handle::new("asd");
        assert_eq!(h.get(), "asd");
    }

    #[test]
    fn test_same() {
        let h1 = Handle::new("asd");
        let h2 = Handle::new("asd");
        assert_eq!(h1, h2);
        assert_eq!(h1.get(), "asd");
        assert_eq!(h2.get(), "asd");
    }

    #[test]
    fn test_not_same() {
        let h1 = Handle::new("asd");
        let h2 = Handle::new("123");
        assert_ne!(h1, h2);
        assert_eq!(h1.get(), "asd");
        assert_eq!(h2.get(), "123");
    }

    #[test]
    #[ignore]
    fn test_pool_gc() {
        assert_eq!(POOL.len(), 0);
        Handle::new("asd");
        assert_eq!(POOL.len(), 1);
        let h = Handle::new("123");
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
                    let a = Handle::from_box(i.to_string().into_boxed_str());
                    let v: Vec<_> = (0..100)
                        .into_iter()
                        .map(|_| spawn(move || Handle::from_box(i.to_string().into_boxed_str())))
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
                        .map(|_| spawn(move || Handle::from_box(i.to_string().into_boxed_str())))
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
