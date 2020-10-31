use std::{
    borrow::{Borrow, Cow},
    convert::identity,
    error::Error,
    ffi::{OsStr, OsString},
    hash::{self, Hash},
    iter::FromIterator,
    net::ToSocketAddrs,
    ops::{Deref, Index},
    path::{Path, PathBuf},
    rc::Rc,
    slice::SliceIndex,
    str::{self, FromStr},
    string::ParseError,
    sync::Arc,
};

use crate::{
    intern::Interned,
    pool::{Intern, STR_POOL},
    MowStr,
};

/// Immutable Interning String
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IStr(Intern<str>);

impl IStr {
    /// Create a `IStr` from str slice  
    ///
    /// # Example
    /// ```
    /// # use pstr::IStr;
    /// let s = IStr::new("hello world");
    /// ```
    #[inline]
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(STR_POOL.intern(s.as_ref(), Arc::from))
    }

    /// Create a `IStr` from `String`  
    #[inline]
    pub fn from_string(s: String) -> Self {
        Self(STR_POOL.intern(s, Arc::from))
    }

    /// Create a `IStr` from `Box<str>`  
    #[inline]
    pub fn from_boxed(s: Box<str>) -> Self {
        Self(STR_POOL.intern(s, Arc::from))
    }

    /// Create a `IStr` from `Arc<str>`  
    #[inline]
    pub fn from_arc(s: Arc<str>) -> Self {
        Self(STR_POOL.intern(s, identity))
    }

    /// Create a `IStr` from `Rc<str>`  
    #[inline]
    pub fn from_rc(s: Rc<str>) -> Self {
        Self(STR_POOL.intern(s, |s| Arc::from(s.to_string())))
    }

    /// Create a `IStr` from `MowStr`  
    #[inline]
    pub fn from_mow(s: MowStr) -> Self {
        s.into()
    }

    /// Create a `IStr` from custom fn  
    #[inline]
    pub fn from_to_arc<S: AsRef<str>>(s: S, to_arc: impl FnOnce(S) -> Arc<str>) -> Self {
        Self(STR_POOL.intern(s, to_arc))
    }
}

impl IStr {
    /// Extracts a string slice containing the entire `IStr`
    #[inline]
    pub fn as_str(&self) -> &str {
        self.deref()
    }

    /// Clone a boxed string slice containing the entire `IStr`
    #[inline]
    pub fn into_boxed_str(&self) -> Box<str> {
        self.deref().into()
    }

    /// Convert to `MowStr`  
    #[inline]
    pub fn into_mut(&self) -> MowStr {
        MowStr::from(self.clone())
    }
}

unsafe impl Interned for IStr {}

impl Deref for IStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl FromStr for IStr {
    type Err = ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl AsRef<[u8]> for IStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.deref().as_bytes()
    }
}

impl AsRef<str> for IStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.get()
    }
}

impl AsRef<OsStr> for IStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.deref().as_ref()
    }
}

impl AsRef<Path> for IStr {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.deref().as_ref()
    }
}

impl<I: SliceIndex<str>> Index<I> for IStr {
    type Output = <I as SliceIndex<str>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.deref().index(index)
    }
}

impl Hash for IStr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl Borrow<str> for IStr {
    #[inline]
    fn borrow(&self) -> &str {
        self.deref()
    }
}

impl From<&'_ String> for IStr {
    #[inline]
    fn from(s: &'_ String) -> Self {
        Self::new(s)
    }
}

impl From<&'_ str> for IStr {
    #[inline]
    fn from(s: &'_ str) -> Self {
        Self::new(s)
    }
}

impl From<&'_ mut str> for IStr {
    #[inline]
    fn from(s: &'_ mut str) -> Self {
        Self::new(s)
    }
}

impl From<char> for IStr {
    #[inline]
    fn from(c: char) -> Self {
        let mut tmp = [0; 4];
        Self::new(c.encode_utf8(&mut tmp))
    }
}

impl From<Box<str>> for IStr {
    #[inline]
    fn from(s: Box<str>) -> Self {
        Self::from_boxed(s)
    }
}

impl From<Arc<str>> for IStr {
    #[inline]
    fn from(s: Arc<str>) -> Self {
        Self::from_arc(s)
    }
}

impl From<Rc<str>> for IStr {
    #[inline]
    fn from(s: Rc<str>) -> Self {
        Self::from_rc(s)
    }
}

impl<'a> From<Cow<'a, str>> for IStr {
    #[inline]
    fn from(s: Cow<'a, str>) -> Self {
        Self::new(s)
    }
}

impl<'a> FromIterator<&'a char> for IStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl<'a> FromIterator<&'a str> for IStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl FromIterator<Box<str>> for IStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Box<str>>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl<'a> FromIterator<Cow<'a, str>> for IStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Cow<'a, str>>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl FromIterator<String> for IStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl FromIterator<char> for IStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl ToSocketAddrs for IStr {
    type Iter = <str as ToSocketAddrs>::Iter;

    #[inline]
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        ToSocketAddrs::to_socket_addrs(self.deref())
    }
}

impl From<IStr> for Box<str> {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IStr> for Vec<u8> {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IStr> for Arc<str> {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IStr> for Rc<str> {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from(v.deref())
    }
}

impl<'a> From<IStr> for Cow<'a, str> {
    #[inline]
    fn from(v: IStr) -> Self {
        Cow::Owned(v.to_string())
    }
}

impl<'a> From<&'a IStr> for Cow<'a, str> {
    #[inline]
    fn from(v: &'a IStr) -> Self {
        Cow::Borrowed(v.deref())
    }
}

impl ToString for IStr {
    fn to_string(&self) -> String {
        self.deref().to_string()
    }
}

impl From<IStr> for Box<dyn Error> {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IStr> for Box<dyn Error + Send + Sync> {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IStr> for OsString {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IStr> for PathBuf {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IStr> for String {
    #[inline]
    fn from(v: IStr) -> Self {
        v.to_string()
    }
}

impl From<String> for IStr {
    fn from(v: String) -> Self {
        Self::from_string(v)
    }
}

impl PartialEq<str> for IStr {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

impl PartialEq<&str> for IStr {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<String> for IStr {
    fn eq(&self, other: &String) -> bool {
        self.deref() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let s = IStr::new("asd");
        assert_eq!(s, "asd");
    }

    #[test]
    fn test_2() {
        let a = IStr::new("asd");
        let b = IStr::new("asd");
        assert_eq!(a, b);
    }

    #[test]
    fn test_3() {
        let a = IStr::new("asd");
        let b = IStr::new("123");
        assert_ne!(a, b);
    }
}
