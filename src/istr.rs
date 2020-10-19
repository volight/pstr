use std::{
    borrow::{Borrow, Cow},
    error::Error,
    ffi::{OsStr, OsString},
    hash::{self, Hash},
    intrinsics::transmute,
    iter::FromIterator,
    net::ToSocketAddrs,
    ops::{Deref, Index},
    path::{Path, PathBuf},
    rc::Rc,
    slice::SliceIndex,
    str::{self, from_utf8, from_utf8_unchecked, FromStr, Utf8Error},
    string::{FromUtf16Error, ParseError},
    sync::Arc,
};

use crate::pool::Handle;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IStr(Handle);

impl IStr {
    #[inline]
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(Handle::new(s.as_ref().as_bytes()))
    }

    #[inline]
    pub fn from_utf8(s: impl AsRef<[u8]>) -> Result<Self, Utf8Error> {
        from_utf8(s.as_ref()).map(Self::new)
    }

    #[inline]
    pub fn from_utf16(s: impl AsRef<[u16]>) -> Result<Self, FromUtf16Error> {
        Ok(Self::from_string(String::from_utf16(s.as_ref())?))
    }

    #[inline]
    pub fn from_string(s: String) -> Self {
        Self::from_boxed_str(s.into_boxed_str())
    }

    #[inline]
    pub fn from_boxed_str(s: Box<str>) -> Self {
        Self(Handle::from_box(s.into()))
    }

    #[inline]
    pub fn from_arc_str(s: Arc<str>) -> Self {
        unsafe { Self::from_raw_arc(transmute(s)) }
    }

    #[inline]
    pub unsafe fn from_raw_arc(s: Arc<[u8]>) -> Self {
        Self(Handle::from_arc(s))
    }

    #[inline]
    pub unsafe fn from_raw(s: *const [u8]) -> Self {
        Self(Handle::from_raw(s))
    }

    #[inline]
    pub unsafe fn from_utf8_unchecked(bytes: impl AsRef<[u8]>) -> Self {
        Self::new(from_utf8_unchecked(bytes.as_ref()))
    }
}

impl IStr {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.deref()
    }

    #[inline]
    pub fn as_arc(&self) -> Arc<str> {
        unsafe { transmute(self.0.get_arc()) }
    }

    #[inline]
    pub fn into_boxed_str(self) -> Box<str> {
        self.deref().into()
    }
}

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
        self.0.get()
    }
}

impl AsRef<str> for IStr {
    #[inline]
    fn as_ref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.0.get()) }
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
        Self::from_boxed_str(s)
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
        unsafe { transmute(v.0.get_arc()) }
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
