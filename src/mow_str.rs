use std::{
    borrow::{Borrow, BorrowMut, Cow},
    error::Error,
    ffi::{OsStr, OsString},
    fmt::Write,
    hash::{self, Hash},
    iter::{Extend, FromIterator},
    net::ToSocketAddrs,
    ops::{Add, AddAssign, Deref, DerefMut, Index, IndexMut, RangeBounds},
    path::{Path, PathBuf},
    rc::Rc,
    slice::SliceIndex,
    str::{self, from_utf8, from_utf8_unchecked, FromStr, Utf8Error},
    string::{Drain, FromUtf16Error, ParseError},
    sync::Arc,
};

use crate::{
    intern::{Interned, Muterned},
    IStr,
};

#[derive(Debug, Eq, Ord, PartialOrd)]
enum MowStrInteral {
    I(IStr),
    M(Option<String>),
}

impl PartialEq for MowStrInteral {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match self {
            MowStrInteral::I(s) => match other {
                MowStrInteral::I(o) => s == o,
                MowStrInteral::M(o) => o.as_ref().unwrap() == s.deref(),
            },
            MowStrInteral::M(s) => match other {
                MowStrInteral::I(o) => s.as_ref().unwrap() == o.deref(),
                MowStrInteral::M(o) => s == o,
            },
        }
    }
}

/// Mutable on Write Interning Pool String
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MowStr(MowStrInteral);

impl MowStr {
    #[inline]
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(MowStrInteral::I(IStr::new(s)))
    }

    #[inline]
    pub fn new_mut(s: impl Into<String>) -> Self {
        Self(MowStrInteral::M(Some(s.into())))
    }

    #[inline]
    pub fn mut_empty() -> Self {
        Self::new_mut(String::new())
    }

    #[inline]
    pub fn mut_with_capacity(capacity: usize) -> Self {
        Self::new_mut(String::with_capacity(capacity))
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
    pub fn from_string_mut(s: String) -> Self {
        Self(MowStrInteral::M(Some(s)))
    }

    #[inline]
    pub fn from_boxed_str(s: Box<str>) -> Self {
        Self(MowStrInteral::I(IStr::from_boxed_str(s)))
    }

    #[inline]
    pub fn from_boxed_str_mut(s: Box<str>) -> Self {
        Self(MowStrInteral::M(Some(s.to_string())))
    }

    #[inline]
    pub fn from_istr(s: IStr) -> Self {
        Self(MowStrInteral::I(s))
    }

    #[inline]
    pub fn from_istr_mut(s: IStr) -> Self {
        Self(MowStrInteral::M(Some(s.to_string())))
    }

    #[inline]
    pub unsafe fn from_utf8_unchecked(bytes: impl AsRef<[u8]>) -> Self {
        Self::new(from_utf8_unchecked(bytes.as_ref()))
    }
}

impl MowStr {
    #[inline]
    pub fn intern(&mut self) {
        let s = match &mut self.0 {
            MowStrInteral::I(_) => return,
            MowStrInteral::M(s) => s.take().unwrap(),
        };
        *self = Self::from_string(s);
    }

    #[inline]
    pub fn to_mut(&mut self) {
        let s = match &mut self.0 {
            MowStrInteral::I(v) => v.to_string(),
            MowStrInteral::M(_) => return,
        };
        *self = Self::from_string_mut(s);
    }

    #[inline]
    pub fn mutdown(&mut self) -> &mut String {
        self.to_mut();
        match &mut self.0 {
            MowStrInteral::I(_) => panic!("never"),
            MowStrInteral::M(v) => v.as_mut().unwrap(),
        }
    }

    #[inline]
    pub fn is_interned(&self) -> bool {
        matches!(&self.0, MowStrInteral::I(_))
    }

    #[inline]
    pub fn is_mutable(&self) -> bool {
        matches!(&self.0, MowStrInteral::M(_))
    }
}

impl MowStr {
    #[inline]
    pub fn ref_str(&self) -> &str {
        self.deref()
    }

    #[inline]
    pub fn mut_str(&mut self) -> &mut str {
        self.as_mut()
    }

    #[inline]
    pub fn mut_string(&mut self) -> &mut String {
        self.as_mut()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.deref()
    }

    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        self.mut_str()
    }

    #[inline]
    pub fn as_mut_string(&mut self) -> &mut String {
        self.mut_string()
    }

    #[inline]
    pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        self.mutdown().as_mut_vec()
    }

    #[inline]
    pub fn into_string(self) -> String {
        match self.0 {
            MowStrInteral::I(v) => v.to_string(),
            MowStrInteral::M(v) => v.unwrap(),
        }
    }

    #[inline]
    pub fn into_boxed_str(self) -> Box<str> {
        match self.0 {
            MowStrInteral::I(v) => v.into_boxed_str(),
            MowStrInteral::M(v) => v.unwrap().into_boxed_str(),
        }
    }
}

impl MowStr {
    #[inline]
    pub fn push_str(&mut self, string: impl AsRef<str>) {
        self.mutdown().push_str(string.as_ref())
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.mutdown().reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.mutdown().reserve_exact(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.mutdown().shrink_to_fit()
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        self.mutdown().push(ch)
    }

    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.mutdown().truncate(new_len)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        self.mutdown().pop()
    }

    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        self.mutdown().remove(idx)
    }

    #[inline]
    pub fn retain<F: FnMut(char) -> bool>(&mut self, f: F) {
        self.mutdown().retain(f)
    }

    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.mutdown().insert(idx, ch)
    }

    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        self.mutdown().insert_str(idx, string)
    }

    #[inline]
    pub fn split_off(&mut self, at: usize) -> MowStr {
        Self::from_string_mut(self.mutdown().split_off(at))
    }

    #[inline]
    pub fn clear(&mut self) {
        self.mutdown().clear()
    }

    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_> {
        self.mutdown().drain(range)
    }

    #[inline]
    pub fn replace_range<R: RangeBounds<usize>>(&mut self, range: R, replace_with: &str) {
        self.mutdown().replace_range(range, replace_with)
    }
}

unsafe impl Interned for MowStr {}
unsafe impl Muterned for MowStr {}

impl Clone for MowStr {
    fn clone(&self) -> Self {
        match &self.0 {
            MowStrInteral::I(v) => Self::from(v.clone()),
            MowStrInteral::M(v) => Self::from_string(v.clone().unwrap()),
        }
    }
}

impl Deref for MowStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for MowStr {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl FromStr for MowStr {
    type Err = ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl AsRef<str> for MowStr {
    #[inline]
    fn as_ref(&self) -> &str {
        match &self.0 {
            MowStrInteral::I(v) => v.as_ref(),
            MowStrInteral::M(v) => v.as_ref().unwrap(),
        }
    }
}

impl AsMut<str> for MowStr {
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        self.mutdown()
    }
}

impl AsMut<String> for MowStr {
    #[inline]
    fn as_mut(&mut self) -> &mut String {
        self.mutdown()
    }
}

impl AsRef<[u8]> for MowStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match &self.0 {
            MowStrInteral::I(v) => v.as_ref(),
            MowStrInteral::M(v) => v.as_ref().unwrap().as_ref(),
        }
    }
}

impl AsRef<OsStr> for MowStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        match &self.0 {
            MowStrInteral::I(v) => v.as_ref(),
            MowStrInteral::M(v) => v.as_ref().unwrap().as_ref(),
        }
    }
}

impl AsRef<Path> for MowStr {
    #[inline]
    fn as_ref(&self) -> &Path {
        match &self.0 {
            MowStrInteral::I(v) => v.as_ref(),
            MowStrInteral::M(v) => v.as_ref().unwrap().as_ref(),
        }
    }
}

impl<I: SliceIndex<str>> Index<I> for MowStr {
    type Output = <I as SliceIndex<str>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.deref().index(index)
    }
}

impl<I: SliceIndex<str>> IndexMut<I> for MowStr {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.deref_mut().index_mut(index)
    }
}

impl Hash for MowStr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl Borrow<str> for MowStr {
    #[inline]
    fn borrow(&self) -> &str {
        self.deref()
    }
}

impl BorrowMut<str> for MowStr {
    #[inline]
    fn borrow_mut(&mut self) -> &mut str {
        self.deref_mut()
    }
}

impl<'a> Extend<&'a char> for MowStr {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
        self.mutdown().extend(iter)
    }
}

impl<'a> Extend<&'a str> for MowStr {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        self.mutdown().extend(iter)
    }
}

impl Extend<Box<str>> for MowStr {
    #[inline]
    fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
        self.mutdown().extend(iter)
    }
}

impl<'a> Extend<Cow<'a, str>> for MowStr {
    #[inline]
    fn extend<T: IntoIterator<Item = Cow<'a, str>>>(&mut self, iter: T) {
        self.mutdown().extend(iter)
    }
}

impl Extend<String> for MowStr {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.mutdown().extend(iter)
    }
}

impl Extend<IStr> for MowStr {
    #[inline]
    fn extend<T: IntoIterator<Item = IStr>>(&mut self, iter: T) {
        let stri = self.mutdown();
        iter.into_iter().for_each(move |s| stri.push_str(&s))
    }
}

impl Extend<MowStr> for MowStr {
    #[inline]
    fn extend<T: IntoIterator<Item = MowStr>>(&mut self, iter: T) {
        let stri = self.mutdown();
        iter.into_iter().for_each(move |s| stri.push_str(&s))
    }
}

impl Add<&str> for MowStr {
    type Output = MowStr;

    #[inline]
    fn add(mut self, rhs: &str) -> Self::Output {
        self.mutdown().push_str(rhs);
        self
    }
}

impl AddAssign<&str> for MowStr {
    #[inline]
    fn add_assign(&mut self, rhs: &str) {
        self.mutdown().push_str(rhs);
    }
}

impl From<&String> for MowStr {
    #[inline]
    fn from(s: &String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for MowStr {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<&mut str> for MowStr {
    #[inline]
    fn from(s: &mut str) -> Self {
        Self::new(s)
    }
}

impl From<String> for MowStr {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<Box<str>> for MowStr {
    #[inline]
    fn from(s: Box<str>) -> Self {
        Self::from_boxed_str(s)
    }
}

impl<'a> From<Cow<'a, str>> for MowStr {
    #[inline]
    fn from(s: Cow<'a, str>) -> Self {
        Self::from_string(s.into_owned())
    }
}

impl From<char> for MowStr {
    #[inline]
    fn from(c: char) -> Self {
        let mut tmp = [0; 4];
        Self::new(c.encode_utf8(&mut tmp))
    }
}

impl ToSocketAddrs for MowStr {
    type Iter = <str as ToSocketAddrs>::Iter;

    #[inline]
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        ToSocketAddrs::to_socket_addrs(self.deref())
    }
}

impl Write for MowStr {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.push(c);
        Ok(())
    }
}

impl ToString for MowStr {
    #[inline]
    fn to_string(&self) -> String {
        match &self.0 {
            MowStrInteral::I(v) => v.to_string(),
            MowStrInteral::M(v) => v.clone().unwrap(),
        }
    }
}

impl From<MowStr> for String {
    #[inline]
    fn from(v: MowStr) -> Self {
        v.into_string()
    }
}

impl<'a> FromIterator<&'a char> for MowStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl<'a> FromIterator<&'a str> for MowStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl FromIterator<Box<str>> for MowStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Box<str>>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl<'a> FromIterator<Cow<'a, str>> for MowStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Cow<'a, str>>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl FromIterator<String> for MowStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl FromIterator<char> for MowStr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self::from_string(String::from_iter(iter))
    }
}

impl From<MowStr> for Box<str> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            MowStrInteral::I(v) => Self::from(v.deref()),
            MowStrInteral::M(v) => Self::from(v.as_deref().unwrap()),
        }
    }
}

impl From<MowStr> for Vec<u8> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            MowStrInteral::I(v) => Self::from(v.deref()),
            MowStrInteral::M(v) => Self::from(v.as_deref().unwrap()),
        }
    }
}

impl From<MowStr> for Arc<str> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            MowStrInteral::I(v) => Self::from(v.clone()),
            MowStrInteral::M(v) => Self::from(v.clone().unwrap()),
        }
    }
}

impl From<MowStr> for Rc<str> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            MowStrInteral::I(v) => Self::from(v.clone()),
            MowStrInteral::M(v) => Self::from(v.clone().unwrap()),
        }
    }
}

impl<'a> From<MowStr> for Cow<'a, str> {
    #[inline]
    fn from(v: MowStr) -> Self {
        Cow::Owned(v.to_string())
    }
}

impl<'a> From<&'a MowStr> for Cow<'a, str> {
    #[inline]
    fn from(v: &'a MowStr) -> Self {
        Cow::Borrowed(v.deref())
    }
}

impl From<MowStr> for Box<dyn Error> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            MowStrInteral::I(v) => Self::from(v.clone()),
            MowStrInteral::M(v) => Self::from(v.clone().unwrap()),
        }
    }
}

impl From<MowStr> for Box<dyn Error + Send + Sync> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            MowStrInteral::I(v) => Self::from(v.clone()),
            MowStrInteral::M(v) => Self::from(v.clone().unwrap()),
        }
    }
}

impl From<MowStr> for OsString {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            MowStrInteral::I(v) => Self::from(v.deref()),
            MowStrInteral::M(v) => Self::from(v.as_ref().unwrap()),
        }
    }
}

impl From<MowStr> for PathBuf {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            MowStrInteral::I(v) => Self::from(v.deref()),
            MowStrInteral::M(v) => Self::from(v.as_ref().unwrap()),
        }
    }
}

impl From<IStr> for MowStr {
    #[inline]
    fn from(v: IStr) -> Self {
        Self::from_istr(v)
    }
}

impl From<MowStr> for IStr {
    fn from(v: MowStr) -> Self {
        match v.0 {
            MowStrInteral::I(v) => v,
            MowStrInteral::M(v) => Self::from_string(v.unwrap()),
        }
    }
}
