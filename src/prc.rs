use std::{
    borrow::Borrow,
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
}

unsafe impl Send for PrcInner {}
unsafe impl Sync for PrcInner {}

#[derive(Debug)]
pub struct Prc<T: ?Sized> {
    inner: NonNull<PrcInner>,
    data: NonNull<T>,
    _phantom: PhantomData<(PrcInner, T)>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Prc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Prc<T> {}

impl<T: ?Sized> Prc<T> {
    #[inline]
    fn from_inner(inner: NonNull<PrcInner>, data: NonNull<T>) -> Self {
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

    #[inline]
    pub fn from_box(data: Box<T>) -> Self {
        let inner = Box::new(PrcInner {
            strong: atomic::AtomicUsize::new(1),
        });
        Self::from_inner(Box::leak(inner).into(), Box::leak(data).into())
    }
}

impl<T: ?Sized> Prc<T> {
    #[inline]
    pub fn inner_ptr_usize(&self) -> usize {
        self.inner.as_ptr() as usize
    }

    #[inline]
    fn inner(&self) -> &PrcInner {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    fn data(&self) -> &T {
        unsafe { self.data.as_ref() }
    }

    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong.load(Ordering::SeqCst)
    }
}

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

impl<T: ?Sized> Clone for Prc<T> {
    fn clone(&self) -> Self {
        let old_size = self.inner().strong.fetch_add(1, Ordering::Relaxed);
        if old_size > MAX_REFCOUNT {
            abort();
        }
        Self::from_inner(self.inner, self.data)
    }
}

impl<T: ?Sized> Deref for Prc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl<T: ?Sized> Prc<T> {
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        drop(Box::from_raw(self.inner.as_mut()));
        drop(Box::from_raw(self.data.as_mut()));
    }
}

impl<T: ?Sized> Drop for Prc<T> {
    fn drop(&mut self) {
        if self.inner().strong.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        atomic::fence(Ordering::Acquire);

        unsafe { self.drop_slow() };
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Prc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data()
    }
}

impl<T: ?Sized + Eq> Eq for Prc<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for Prc<T> {
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

impl<T: ?Sized + Ord> Ord for Prc<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data().cmp(other.data())
    }
}

impl<T: ?Sized + Default> Default for Prc<T> {
    fn default() -> Self {
        Prc::from_box(Default::default())
    }
}

impl<T: ?Sized + Hash> Hash for Prc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.data(), state)
    }
}

impl<T: ?Sized> AsRef<T> for Prc<T> {
    fn as_ref(&self) -> &T {
        self.data()
    }
}

impl<T: ?Sized> Borrow<T> for Prc<T> {
    fn borrow(&self) -> &T {
        self.data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let prc = Prc::from_box(Box::new(1));
        assert_eq!(*prc, 1)
    }

    struct Foo(pub usize);
    impl Drop for Foo {
        fn drop(&mut self) {
            println!("drop foo");
        }
    }

    #[test]
    fn test_drop() {
        let prc = Prc::from_box(Box::new(Foo(123)));
        drop(prc);
    }

    #[test]
    fn test_clone() {
        let prc = Prc::from_box(Box::new(Foo(123)));
        let prc2 = prc.clone();
        assert_eq!(Prc::<Foo>::strong_count(&prc2), 2);
        drop(prc);
        assert_eq!(Prc::<Foo>::strong_count(&prc2), 1);
        drop(prc2);
    }
}
