//! The String Intern Pool  

use std::sync::Arc;

use dashmap::DashSet;
use once_cell::sync::Lazy;

use crate::prc::Prc;

static POOL: Lazy<Arc<DashSet<Prc<str>>>> = Lazy::new(|| Arc::new(DashSet::new()));

#[derive(Debug, Clone, Eq, Ord, PartialOrd)]
pub(crate) struct Handle(Prc<str>);

impl Handle {
    #[inline]
    pub fn new(slice: &str) -> Self {
        let pool = POOL.clone();
        match pool.get(slice).map(|v| v.key().clone()) {
            Some(v) => Self(v),
            None => {
                let prc = Prc::from_box(Box::from(slice));
                if pool.insert(Clone::clone(&prc)) {
                    Self(prc)
                } else {
                    Self(pool.get(prc.as_ref()).unwrap().key().clone())
                }
            }
        }
    }
    #[inline]
    pub fn from_box(slice: Box<str>) -> Self {
        let pool = POOL.clone();
        match pool.get(slice.as_ref()).map(|v| v.key().clone()) {
            Some(v) => Self(v),
            None => {
                let prc = Prc::from_box(slice);
                if pool.insert(prc.clone()) {
                    Self(prc)
                } else {
                    Self(pool.get(prc.as_ref()).unwrap().key().clone())
                }
            }
        }
    }

    #[inline]
    pub fn get(&self) -> &str {
        self.0.as_ref()
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        self.0.inner_ptr_usize() == other.0.inner_ptr_usize()
    }
}

/// Delete all interning string with reference count == 1 in the pool
pub fn collect_garbage() {
    let pool = POOL.clone();
    pool.retain(|prc| Prc::<str>::strong_count(prc) > 1)
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
        let pool = POOL.clone();
        assert_eq!(pool.len(), 0);
        Handle::new("asd");
        assert_eq!(pool.len(), 1);
        let h = Handle::new("123");
        assert_eq!(pool.len(), 2);
        collect_garbage();
        assert_eq!(pool.len(), 1);
        drop(h);
        assert_eq!(pool.len(), 1);
        collect_garbage();
        assert_eq!(pool.len(), 0);
    }
}
