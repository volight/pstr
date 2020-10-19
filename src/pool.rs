use once_cell::sync::Lazy;
use std::collections::hash_set::HashSet;
use std::sync::{Arc, Mutex};

static POOL: Lazy<Mutex<HashSet<Arc<[u8]>>>> = Lazy::new(|| Mutex::new(HashSet::new()));

#[derive(Debug, Clone, Eq, Ord, PartialOrd)]
pub struct Handle(Arc<[u8]>);

impl Handle {
    #[inline]
    pub fn new(slice: &[u8]) -> Self {
        let mut pool = POOL.lock().unwrap();
        match pool.get(slice).cloned() {
            Some(v) => Self(v),
            None => {
                let s = Self(Arc::from(slice));
                let v = s.0.clone();
                pool.insert(v);
                s
            }
        }
    }

    #[inline]
    pub unsafe fn from_raw(ptr: *const [u8]) -> Self {
        let mut pool = POOL.lock().unwrap();
        match pool.get(&*ptr).cloned() {
            Some(v) => Self(v),
            None => {
                let s = Self(Arc::from_raw(ptr));
                let v = s.0.clone();
                pool.insert(v);
                s
            }
        }
    }

    #[inline]
    pub fn from_arc(arc: Arc<[u8]>) -> Self {
        let mut pool = POOL.lock().unwrap();
        match pool.get(arc.as_ref()).cloned() {
            Some(v) => Self(v),
            None => {
                let s = Self(arc);
                let v = s.0.clone();
                pool.insert(v);
                s
            }
        }
    }

    #[inline]
    pub fn from_box(slice: Box<[u8]>) -> Self {
        let arc = Arc::from(slice);
        Self::from_arc(arc)
    }

    // #[inline]
    // pub fn from_iter(item: impl Iterator<Item = u8>) -> Self {
    //     let arc = item.collect::<Arc<[u8]>>();
    //     Self::from_arc(arc)
    // }

    #[inline]
    pub fn get(&self) -> &[u8] {
        self.0.as_ref()
    }

    #[inline]
    pub fn get_arc(&self) -> Arc<[u8]> {
        self.0.clone()
    }
}

impl PartialEq for Handle {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}
