use std::{collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;

use crate::prc::Prc;

static POOL: Lazy<Mutex<HashMap<u64, Prc>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Handle(Prc);

impl Handle {
    #[inline]
    pub fn new(slice: &[u8]) -> Self {
        let hash = Prc::make_hash(slice);
        let mut pool = POOL.lock().unwrap();
        match pool.get(&hash) {
            Some(v) => Self(v.clone()),
            None => {
                let s = Self(Prc::from_slice(slice));
                let v = s.0.clone();
                pool.insert(hash, v);
                s
            }
        }
    }

    #[inline]
    pub fn from_box(slice: Box<[u8]>) -> Self {
        let hash = Prc::make_hash(&*slice);
        let mut pool = POOL.lock().unwrap();
        match pool.get(&hash) {
            Some(v) => Self(v.clone()),
            None => {
                let s = Self(Prc::from_box(slice));
                let v = s.0.clone();
                pool.insert(hash, v);
                s
            }
        }
    }

    #[inline]
    pub fn get(&self) -> &[u8] {
        self.0.as_ref()
    }
}
