use std::{
    borrow::Borrow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    process::abort,
    ptr::NonNull,
    sync::atomic,
    sync::atomic::Ordering,
};

#[repr(C)]
struct PrcInner {
    strong: atomic::AtomicUsize,
    hash: u64,
}

unsafe impl Send for PrcInner {}
unsafe impl Sync for PrcInner {}

#[derive(Debug)]
pub struct Prc {
    inner: NonNull<PrcInner>,
    data: NonNull<[u8]>,
    _phantom: PhantomData<(PrcInner, [u8])>,
}

unsafe impl Send for Prc {}
unsafe impl Sync for Prc {}

impl Prc {
    #[inline]
    fn from_inner(inner: NonNull<PrcInner>, data: NonNull<[u8]>) -> Self {
        Self {
            inner,
            data,
            _phantom: PhantomData,
        }
    }

    // #[inline]
    // unsafe fn from_ptr(pinnertr: *mut PrcInner, data: *mut [u8]) -> Self {
    //     Self::from_inner(
    //         NonNull::new_unchecked(pinnertr),
    //         NonNull::new_unchecked(data),
    //     )
    // }
}

impl Prc {
    #[inline]
    fn from_box_with_hash(data: Box<[u8]>, hash: u64) -> Self {
        let inner = Box::new(PrcInner {
            strong: atomic::AtomicUsize::new(1),
            hash,
        });
        Self::from_inner(Box::leak(inner).into(), Box::leak(data).into())
    }
}

impl Prc {
    #[inline]
    pub fn make_hash(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::default();
        data.hash(&mut hasher);
        hasher.finish()
    }
}

impl Prc {
    #[inline]
    pub fn from_box(data: Box<[u8]>) -> Self {
        let hash = Self::make_hash(&*data);
        Self::from_box_with_hash(data, hash)
    }

    #[inline]
    pub fn from_slice(data: &[u8]) -> Self {
        Self::from_box(data.into())
    }
}

impl Prc {
    #[inline]
    fn inner(&self) -> &PrcInner {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    fn data(&self) -> &[u8] {
        unsafe { self.data.as_ref() }
    }

    // #[inline]
    // pub fn strong_count(this: &Self) -> usize {
    //     this.inner().strong.load(Ordering::SeqCst)
    // }
}

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

impl Clone for Prc {
    fn clone(&self) -> Self {
        let old_size = self.inner().strong.fetch_add(1, Ordering::Relaxed);
        if old_size > MAX_REFCOUNT {
            abort();
        }
        Self::from_inner(self.inner, self.data)
    }
}

impl Deref for Prc {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl Prc {
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        drop(Box::from_raw(self.inner.as_mut()));
        drop(Box::from_raw(self.data.as_mut()));
    }
}

impl Drop for Prc {
    fn drop(&mut self) {
        if self.inner().strong.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        atomic::fence(Ordering::Acquire);

        unsafe { self.drop_slow() };
    }
}

impl PartialEq for Prc {
    fn eq(&self, other: &Self) -> bool {
        self.get_hash() == other.get_hash()
    }
}

impl Eq for Prc {}

impl PartialOrd for Prc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data().partial_cmp(other.data())
    }

    fn lt(&self, other: &Self) -> bool {
        self.data().lt(other.data())
    }

    fn le(&self, other: &Self) -> bool {
        self.data().le(other.data())
    }

    fn gt(&self, other: &Self) -> bool {
        self.data().gt(other.data())
    }

    fn ge(&self, other: &Self) -> bool {
        self.data().ge(other.data())
    }
}

impl Ord for Prc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data().cmp(other.data())
    }
}

impl Default for Prc {
    fn default() -> Self {
        Prc::from_box(Default::default())
    }
}

impl Hash for Prc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.inner().hash, state)
    }
}

impl Prc {
    #[inline]
    pub fn get_hash(&self) -> u64 {
        self.inner().hash
    }
}

impl AsRef<[u8]> for Prc {
    fn as_ref(&self) -> &[u8] {
        self.data()
    }
}

impl AsRef<u64> for Prc {
    fn as_ref(&self) -> &u64 {
        &self.inner().hash
    }
}

impl Borrow<[u8]> for Prc {
    fn borrow(&self) -> &[u8] {
        self.data()
    }
}
