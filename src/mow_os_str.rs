use std::{
    borrow::{Borrow, BorrowMut},
    ffi::OsStr,
    ffi::OsString,
    hash::{self, Hash},
    ops::{Add, AddAssign, Deref, DerefMut},
    path::Path,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use crate::{
    ffi::IOsStr,
    intern::{Interned, Muterned},
};

#[derive(Debug, Eq, Ord, PartialOrd)]
enum MowOsStrInner {
    I(IOsStr),
    M(Option<OsString>),
}

type Inner = MowOsStrInner;

impl PartialEq for MowOsStrInner {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::I(s) => match other {
                Self::I(o) => s == o,
                Self::M(o) => o.as_ref().unwrap() == s.deref(),
            },
            Self::M(s) => match other {
                Self::I(o) => s.as_ref().unwrap() == o.deref(),
                Self::M(o) => s == o,
            },
        }
    }
}

/// Mutable on Write Interning OsString  
///
/// It will be auto switch to mutable when do modify operate  
///
/// Can call `.intern()` to save into intern pool
///
/// # Example
/// ```
/// # use pstr::ffi::MowOsStr;
/// let mut s = MowOsStr::new("hello");
/// assert!(s.is_interned());
///
/// s.push(" ");
/// assert!(s.is_mutable());
///
/// s.mutdown().push("world");
/// assert_eq!(s, "hello world");
///
/// s.intern();
/// assert!(s.is_interned());
/// ```
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MowOsStr(Inner);

impl MowOsStr {
    /// Create a `MowOsStr` from OsStr slice  
    ///
    /// # Example
    /// ```
    /// # use pstr::ffi::MowOsStr;
    /// let s = MowOsStr::new("hello world");
    /// ```
    #[inline]
    pub fn new(s: impl AsRef<OsStr>) -> Self {
        Self(Inner::I(IOsStr::new(s)))
    }

    /// Create a `MowOsStr` from OsStr slice with mutable  
    ///
    /// # Example
    /// ```
    /// # use pstr::ffi::MowOsStr;
    /// let s = MowOsStr::new_mut("hello world");
    /// assert!(s.is_mutable());
    /// ```
    #[inline]
    pub fn new_mut(s: impl Into<OsString>) -> Self {
        Self(Inner::M(Some(s.into())))
    }

    /// Create a new empty `MowOsStr` with mutable  
    ///
    /// # Example
    /// ```
    /// # use pstr::ffi::MowOsStr;
    /// let s = MowOsStr::mut_empty();
    /// assert!(s.is_mutable());
    /// ```
    #[inline]
    pub fn mut_empty() -> Self {
        Self::new_mut(OsString::new())
    }

    /// Create a new empty `MowOsStr` with a particular capacity and mutable  
    #[inline]
    pub fn mut_with_capacity(capacity: usize) -> Self {
        Self::new_mut(OsString::with_capacity(capacity))
    }

    /// Create a `MowOsStr` from `String`  
    #[inline]
    pub fn from_os_string(s: OsString) -> Self {
        Self(Inner::I(IOsStr::from_os_string(s)))
    }

    /// Create a `MowOsStr` from `String` with mutable  
    #[inline]
    pub fn from_os_string_mut(s: OsString) -> Self {
        Self(Inner::M(Some(s)))
    }

    /// Create a `MowOsStr` from `Box<OsStr>`  
    #[inline]
    pub fn from_boxed(s: Box<OsStr>) -> Self {
        Self(Inner::I(IOsStr::from_boxed(s)))
    }

    /// Create a `MowOsStr` from `Arc<OsStr>`  
    #[inline]
    pub fn from_arc(s: Arc<OsStr>) -> Self {
        Self(Inner::I(IOsStr::from_arc(s)))
    }

    /// Create a `MowOsStr` from `Rc<OsStr>`  
    #[inline]
    pub fn from_rc(s: Rc<OsStr>) -> Self {
        Self(Inner::I(IOsStr::from_rc(s)))
    }

    /// Create a `MowOsStr` from `IOsStr`  
    #[inline]
    pub fn from_i_os_str(s: IOsStr) -> Self {
        Self(Inner::I(s))
    }

    /// Create a `MowOsStr` from custom fn  
    #[inline]
    pub fn from_to_arc<S: AsRef<OsStr>>(s: S, to_arc: impl FnOnce(S) -> Arc<OsStr>) -> Self {
        Self(Inner::I(IOsStr::from_to_arc(s, to_arc)))
    }
}

impl MowOsStr {
    /// Save the current state to the intern pool  
    /// Do nothing if already in the pool  
    #[inline]
    pub fn intern(&mut self) {
        let s = match &mut self.0 {
            Inner::I(_) => return,
            MowOsStrInner::M(s) => s.take().unwrap(),
        };
        *self = Self::from_os_string(s);
    }

    /// Get a mutable clone of the string on the pool  
    /// Do nothing if already mutable  
    #[inline]
    pub fn to_mut(&mut self) {
        let s = match &mut self.0 {
            Inner::I(v) => v.to_os_string(),
            Inner::M(_) => return,
        };
        *self = Self::from_os_string_mut(s);
    }

    /// Switch to mutable and return a mutable reference  
    #[inline]
    pub fn mutdown(&mut self) -> &mut OsString {
        self.to_mut();
        match &mut self.0 {
            Inner::I(_) => panic!("never"),
            Inner::M(v) => v.as_mut().unwrap(),
        }
    }

    /// Do nothing if already mutable  
    #[inline]
    pub fn to_mut_by(&mut self, f: impl FnOnce(&mut IOsStr) -> OsString) {
        let s = match &mut self.0 {
            Inner::I(v) => f(v),
            Inner::M(_) => return,
        };
        *self = Self::from_os_string_mut(s);
    }

    /// Swap internal OsString  
    /// Return `None` if self is interned  
    pub fn swap_mut(&mut self, s: OsString) -> Option<OsString> {
        let r = match &mut self.0 {
            Inner::I(_) => None,
            MowOsStrInner::M(s) => Some(s.take().unwrap()),
        };
        *self = Self::from_os_string_mut(s);
        r
    }

    /// Swap internal OsString when self is mutable  
    /// Do nothing if self is interned  
    /// Return `None` if self is interned  
    pub fn try_swap_mut(&mut self, s: OsString) -> Option<OsString> {
        let r = match &mut self.0 {
            Inner::I(_) => None,
            MowOsStrInner::M(s) => Some(s.take().unwrap()),
        };
        if r.is_some() {
            *self = Self::from_os_string_mut(s);
        }
        r
    }

    /// Check if it is in intern pool  
    #[inline]
    pub fn is_interned(&self) -> bool {
        matches!(&self.0, Inner::I(_))
    }

    /// Check if it is mutable  
    #[inline]
    pub fn is_mutable(&self) -> bool {
        matches!(&self.0, Inner::M(_))
    }

    /// Try get `IOsStr`
    #[inline]
    pub fn try_istr(&self) -> Option<&IOsStr> {
        match &self.0 {
            Inner::I(v) => Some(v),
            Inner::M(_) => None,
        }
    }

    /// Try get `OsString`
    #[inline]
    pub fn try_string(&self) -> Option<&OsString> {
        match &self.0 {
            Inner::I(_) => None,
            Inner::M(v) => Some(v.as_ref().unwrap()),
        }
    }
}

impl MowOsStr {
    /// Get `&str`  
    #[inline]
    pub fn ref_os_str(&self) -> &OsStr {
        self.deref()
    }

    /// Get `&mut str`  
    #[inline]
    pub fn mut_os_str(&mut self) -> &mut OsStr {
        self.as_mut()
    }

    /// Get `&mut String`  
    #[inline]
    pub fn mut_os_string(&mut self) -> &mut OsString {
        self.as_mut()
    }

    /// Extracts a string slice containing the entire `MowStr`
    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        self.deref()
    }

    /// Switch to mutable and returns a mutable string slice.
    #[inline]
    pub fn as_mut_os_str(&mut self) -> &mut OsStr {
        self.mut_os_str()
    }

    /// Switch to mutable and returns a mutable `String` reference
    #[inline]
    pub fn as_mut_os_string(&mut self) -> &mut OsString {
        self.mut_os_string()
    }

    /// Convert to `String`  
    #[inline]
    pub fn into_os_string(self) -> OsString {
        match self.0 {
            Inner::I(v) => v.to_os_string(),
            Inner::M(v) => v.unwrap(),
        }
    }

    /// Convert to `Box<str>`  
    #[inline]
    pub fn into_boxed_os_str(self) -> Box<OsStr> {
        match self.0 {
            Inner::I(v) => v.into_boxed_os_str(),
            Inner::M(v) => v.unwrap().into_boxed_os_str(),
        }
    }
}

impl MowOsStr {
    /// Extends the string with the given `&OsStr` slice.
    #[inline]
    pub fn push(&mut self, s: impl AsRef<OsStr>) {
        self.mutdown().push(s)
    }

    /// Truncates the `MowOsStr` to zero length.
    #[inline]
    pub fn clear(&mut self) {
        self.mutdown().clear();
    }

    /// Reserves capacity for at least `additional` more capacity to be inserted in the given `MowOsStr`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.mutdown().reserve(additional)
    }

    /// Reserves the minimum capacity for exactly `additional` more capacity to be inserted in the given `MowOsStr`. Does nothing if the capacity is already sufficient.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.mutdown().reserve_exact(additional)
    }

    /// Shrinks the capacity of the `MowOsStr` to match its length.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.mutdown().shrink_to_fit()
    }
}

unsafe impl Interned for MowOsStr {}
unsafe impl Muterned for MowOsStr {}

impl Clone for MowOsStr {
    fn clone(&self) -> Self {
        match &self.0 {
            Inner::I(v) => Self::from_i_os_str(v.clone()),
            Inner::M(v) => Self::from_os_string(v.clone().unwrap()),
        }
    }
}

impl Deref for MowOsStr {
    type Target = OsStr;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for MowOsStr {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl AsRef<OsStr> for MowOsStr {
    fn as_ref(&self) -> &OsStr {
        match &self.0 {
            Inner::I(v) => v.as_ref(),
            Inner::M(v) => v.as_ref().unwrap(),
        }
    }
}

impl AsMut<OsStr> for MowOsStr {
    fn as_mut(&mut self) -> &mut OsStr {
        self.mutdown()
    }
}

impl AsMut<OsString> for MowOsStr {
    #[inline]
    fn as_mut(&mut self) -> &mut OsString {
        self.mutdown()
    }
}

impl AsRef<Path> for MowOsStr {
    #[inline]
    fn as_ref(&self) -> &Path {
        match &self.0 {
            Inner::I(v) => v.as_ref(),
            Inner::M(v) => v.as_ref().unwrap().as_ref(),
        }
    }
}

impl Hash for MowOsStr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl Borrow<OsStr> for MowOsStr {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self.deref()
    }
}

impl BorrowMut<OsStr> for MowOsStr {
    #[inline]
    fn borrow_mut(&mut self) -> &mut OsStr {
        self.deref_mut()
    }
}

impl<T: AsRef<OsStr>> Add<T> for MowOsStr {
    type Output = MowOsStr;

    fn add(mut self, rhs: T) -> Self::Output {
        self.mutdown().push(rhs);
        self
    }
}

impl<T: AsRef<OsStr>> AddAssign<T> for MowOsStr {
    fn add_assign(&mut self, rhs: T) {
        self.mutdown().push(rhs);
    }
}

impl From<&OsString> for MowOsStr {
    fn from(s: &OsString) -> Self {
        Self::new(s)
    }
}

impl From<OsString> for MowOsStr {
    fn from(s: OsString) -> Self {
        Self::from_os_string(s)
    }
}

impl From<&OsStr> for MowOsStr {
    fn from(s: &OsStr) -> Self {
        Self::new(s)
    }
}

impl From<&mut OsStr> for MowOsStr {
    fn from(s: &mut OsStr) -> Self {
        Self::new(s)
    }
}

impl From<Box<OsStr>> for MowOsStr {
    fn from(s: Box<OsStr>) -> Self {
        Self::from_boxed(s)
    }
}

impl From<Arc<OsStr>> for MowOsStr {
    fn from(s: Arc<OsStr>) -> Self {
        Self::from_arc(s)
    }
}

impl From<Rc<OsStr>> for MowOsStr {
    fn from(s: Rc<OsStr>) -> Self {
        Self::from_rc(s)
    }
}

impl From<PathBuf> for MowOsStr {
    fn from(s: PathBuf) -> Self {
        Self::from_os_string(s.into())
    }
}

impl From<MowOsStr> for OsString {
    fn from(v: MowOsStr) -> Self {
        match v.0 {
            MowOsStrInner::I(v) => v.to_os_string(),
            MowOsStrInner::M(v) => v.unwrap(),
        }
    }
}

impl From<MowOsStr> for Box<OsStr> {
    fn from(v: MowOsStr) -> Self {
        v.deref().into()
    }
}

impl From<MowOsStr> for Arc<OsStr> {
    fn from(v: MowOsStr) -> Self {
        match v.0 {
            MowOsStrInner::I(v) => v.into(),
            MowOsStrInner::M(v) => v.unwrap().into(),
        }
    }
}

impl From<MowOsStr> for IOsStr {
    fn from(v: MowOsStr) -> Self {
        match v.0 {
            MowOsStrInner::I(v) => v,
            MowOsStrInner::M(v) => IOsStr::from_os_string(v.unwrap()),
        }
    }
}

impl From<IOsStr> for MowOsStr {
    fn from(v: IOsStr) -> Self {
        Self::from_i_os_str(v)
    }
}

impl PartialEq<OsStr> for MowOsStr {
    fn eq(&self, other: &OsStr) -> bool {
        self.deref() == other
    }
}

impl PartialEq<&OsStr> for MowOsStr {
    fn eq(&self, other: &&OsStr) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<OsString> for MowOsStr {
    fn eq(&self, other: &OsString) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<str> for MowOsStr {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

impl PartialEq<&str> for MowOsStr {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<String> for MowOsStr {
    fn eq(&self, other: &String) -> bool {
        self.deref() == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut s = MowOsStr::new("hello");
        assert!(s.is_interned());

        s.push(" ");
        assert!(s.is_mutable());

        s.mutdown().push("world");
        assert_eq!(s, "hello world");
    }
}
