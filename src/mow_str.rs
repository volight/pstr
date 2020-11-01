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
    str::{self, FromStr},
    string::{Drain, ParseError},
    sync::Arc,
};

use crate::{
    intern::{Interned, Muterned},
    IStr,
};

#[derive(Debug, Eq, Ord, PartialOrd)]
enum MowStrInner {
    I(IStr),
    M(Option<String>),
}

type Inner = MowStrInner;

impl PartialEq for MowStrInner {
    #[inline]
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

/// Mutable on Write Interning String  
///
/// It will be auto switch to mutable when do modify operate  
///
/// Can call `.intern()` to save into intern pool
///
/// # Example
/// ```
/// # use pstr::MowStr;
/// let mut s = MowStr::new("hello");
/// assert!(s.is_interned());
///
/// s.push_str(" ");
/// assert!(s.is_mutable());
///
/// s.mutdown().push_str("world");
/// assert_eq!(s, "hello world");
///
/// s.intern();
/// assert!(s.is_interned());
/// ```
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MowStr(Inner);

impl MowStr {
    /// Create a `MowStr` from str slice  
    ///
    /// # Example
    /// ```
    /// # use pstr::MowStr;
    /// let s = MowStr::new("hello world");
    /// ```
    #[inline]
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(Inner::I(IStr::new(s)))
    }

    /// Create a `MowStr` from str slice with mutable  
    ///
    /// # Example
    /// ```
    /// # use pstr::MowStr;
    /// let s = MowStr::new_mut("hello world");
    /// assert!(s.is_mutable());
    /// ```
    #[inline]
    pub fn new_mut(s: impl Into<String>) -> Self {
        Self(Inner::M(Some(s.into())))
    }

    /// Create a new empty `MowStr` with mutable  
    ///
    /// # Example
    /// ```
    /// # use pstr::MowStr;
    /// let s = MowStr::mut_empty();
    /// assert!(s.is_mutable());
    /// ```
    #[inline]
    pub fn mut_empty() -> Self {
        Self::new_mut(String::new())
    }

    /// Create a new empty `MowStr` with a particular capacity and mutable  
    #[inline]
    pub fn mut_with_capacity(capacity: usize) -> Self {
        Self::new_mut(String::with_capacity(capacity))
    }

    /// Create a `MowStr` from `String`  
    #[inline]
    pub fn from_string(s: String) -> Self {
        Self(Inner::I(IStr::from_string(s)))
    }

    /// Create a `MowStr` from `String` with mutable  
    #[inline]
    pub fn from_string_mut(s: String) -> Self {
        Self(Inner::M(Some(s)))
    }

    /// Create a `MowStr` from `Box<str>`  
    #[inline]
    pub fn from_boxed(s: Box<str>) -> Self {
        Self(Inner::I(IStr::from_boxed(s)))
    }

    /// Create a `MowStr` from `Arc<str>`  
    #[inline]
    pub fn from_arc(s: Arc<str>) -> Self {
        Self(Inner::I(IStr::from_arc(s)))
    }

    /// Create a `MowStr` from `Rc<str>`  
    #[inline]
    pub fn from_rc(s: Rc<str>) -> Self {
        Self(Inner::I(IStr::from_rc(s)))
    }

    /// Create a `MowStr` from `IStr`  
    #[inline]
    pub fn from_istr(s: IStr) -> Self {
        Self(Inner::I(s))
    }

    /// Create a `MowStr` from custom fn  
    #[inline]
    pub fn from_to_arc<S: AsRef<str>>(s: S, to_arc: impl FnOnce(S) -> Arc<str>) -> Self {
        Self(Inner::I(IStr::from_to_arc(s, to_arc)))
    }
}

impl MowStr {
    /// Save the current state to the intern pool  
    /// Do nothing if already in the pool  
    #[inline]
    pub fn intern(&mut self) {
        let s = match &mut self.0 {
            Inner::I(_) => return,
            MowStrInner::M(s) => s.take().unwrap(),
        };
        *self = Self::from_string(s);
    }

    /// Get a mutable clone of the string on the pool  
    /// Do nothing if already mutable  
    #[inline]
    pub fn to_mut(&mut self) {
        let s = match &mut self.0 {
            Inner::I(v) => v.to_string(),
            Inner::M(_) => return,
        };
        *self = Self::from_string_mut(s);
    }

    /// Switch to mutable and return a mutable reference  
    #[inline]
    pub fn mutdown(&mut self) -> &mut String {
        self.to_mut();
        match &mut self.0 {
            Inner::I(_) => panic!("never"),
            Inner::M(v) => v.as_mut().unwrap(),
        }
    }

    /// Do nothing if already mutable  
    #[inline]
    pub fn to_mut_by(&mut self, f: impl FnOnce(&mut IStr) -> String) {
        let s = match &mut self.0 {
            Inner::I(v) => f(v),
            Inner::M(_) => return,
        };
        *self = Self::from_string_mut(s);
    }

    /// Swap internal String  
    /// Return `None` if self is interned  
    pub fn swap_mut(&mut self, s: String) -> Option<String> {
        let r = match &mut self.0 {
            Inner::I(_) => None,
            MowStrInner::M(s) => Some(s.take().unwrap()),
        };
        *self = Self::from_string_mut(s);
        r
    }

    /// Swap internal String when self is mutable  
    /// Do nothing if self is interned  
    /// Return `None` if self is interned  
    pub fn try_swap_mut(&mut self, s: String) -> Option<String> {
        let r = match &mut self.0 {
            Inner::I(_) => None,
            MowStrInner::M(s) => Some(s.take().unwrap()),
        };
        if r.is_some() {
            *self = Self::from_string_mut(s);
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

    /// Try get `IStr`
    #[inline]
    pub fn try_istr(&self) -> Option<&IStr> {
        match &self.0 {
            Inner::I(v) => Some(v),
            Inner::M(_) => None,
        }
    }

    /// Try get `String`
    #[inline]
    pub fn try_string(&self) -> Option<&String> {
        match &self.0 {
            Inner::I(_) => None,
            Inner::M(v) => Some(v.as_ref().unwrap()),
        }
    }
}

impl MowStr {
    /// Get `&str`  
    #[inline]
    pub fn ref_str(&self) -> &str {
        self.deref()
    }

    /// Get `&mut str`  
    #[inline]
    pub fn mut_str(&mut self) -> &mut str {
        self.as_mut()
    }

    /// Get `&mut String`  
    #[inline]
    pub fn mut_string(&mut self) -> &mut String {
        self.as_mut()
    }

    /// Extracts a string slice containing the entire `MowStr`
    #[inline]
    pub fn as_str(&self) -> &str {
        self.deref()
    }

    /// Switch to mutable and returns a mutable string slice.
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        self.mut_str()
    }

    /// Switch to mutable and returns a mutable `String` reference
    #[inline]
    pub fn as_mut_string(&mut self) -> &mut String {
        self.mut_string()
    }

    /// Switch to mutable and returns a mutable `Vec<u8>` reference
    #[inline]
    pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        self.mutdown().as_mut_vec()
    }

    /// Convert to `String`  
    #[inline]
    pub fn into_string(self) -> String {
        match self.0 {
            Inner::I(v) => v.to_string(),
            Inner::M(v) => v.unwrap(),
        }
    }

    /// Convert to `Box<str>`  
    #[inline]
    pub fn into_boxed_str(self) -> Box<str> {
        match self.0 {
            Inner::I(v) => v.into_boxed_str(),
            Inner::M(v) => v.unwrap().into_boxed_str(),
        }
    }
}

impl MowStr {
    /// Appends a given string slice onto the end of this `MowStr`  
    #[inline]
    pub fn push_str(&mut self, string: impl AsRef<str>) {
        self.mutdown().push_str(string.as_ref())
    }

    /// Ensures that this `MowStr`'s capacity is at least `additional` bytes larger than its length.  
    ///
    /// The capacity may be increased by more than `additional` bytes if it chooses, to prevent frequent reallocations.  
    ///
    /// If you do not want this "at least" behavior, see the [`reserve_exact`] method.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows [`usize`].
    ///
    /// [`reserve_exact`]: MowStr::reserve_exact
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.mutdown().reserve(additional)
    }

    /// Ensures that this `MowStr`'s capacity is `additional` bytes
    /// larger than its length.
    ///
    /// Consider using the [`reserve`] method unless you absolutely know
    /// better than the allocator.
    ///
    /// [`reserve`]: MowStr::reserve
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.mutdown().reserve_exact(additional)
    }

    /// Shrinks the capacity of this `MowStr` to match its length.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.mutdown().shrink_to_fit()
    }

    /// Appends the given [`char`] to the end of this `MowStr`.
    #[inline]
    pub fn push(&mut self, ch: char) {
        self.mutdown().push(ch)
    }

    /// Shortens this `MowStr` to the specified length.
    ///
    /// If `new_len` is greater than the string's current length, this has no
    /// effect.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the string
    ///
    /// # Panics
    ///
    /// Panics if `new_len` does not lie on a [`char`] boundary.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.mutdown().truncate(new_len)
    }

    /// Removes the last character from the string buffer and returns it.
    ///
    /// Returns [`None`] if this `MowStr` is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        self.mutdown().pop()
    }

    /// Removes a [`char`] from this `MowStr` at a byte position and returns it.
    ///
    /// This is an *O*(*n*) operation, as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the `MowStr`'s length,
    /// or if it does not lie on a [`char`] boundary.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        self.mutdown().remove(idx)
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// In other words, remove all characters `c` such that `f(c)` returns `false`.
    /// This method operates in place, visiting each character exactly once in the
    /// original order, and preserves the order of the retained characters.
    #[inline]
    pub fn retain<F: FnMut(char) -> bool>(&mut self, f: F) {
        self.mutdown().retain(f)
    }

    /// Inserts a character into this `MowStr` at a byte position.
    ///
    /// This is an *O*(*n*) operation as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the `MowStr`'s length, or if it does not
    /// lie on a [`char`] boundary.
    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.mutdown().insert(idx, ch)
    }

    /// Inserts a string slice into this `MowStr` at a byte position.
    ///
    /// This is an *O*(*n*) operation as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the `MowStr`'s length, or if it does not
    /// lie on a [`char`] boundary.
    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        self.mutdown().insert_str(idx, string)
    }

    /// Splits the string into two at the given index.
    ///
    /// Returns a newly allocated `MowStr`. `self` contains bytes `[0, at)`, and
    /// the returned `MowStr` contains bytes `[at, len)`. `at` must be on the
    /// boundary of a UTF-8 code point.
    ///
    /// Note that the capacity of `self` does not change.
    ///
    /// # Panics
    ///
    /// Panics if `at` is not on a `UTF-8` code point boundary, or if it is beyond the last
    /// code point of the string.
    #[inline]
    pub fn split_off(&mut self, at: usize) -> MowStr {
        Self::from_string_mut(self.mutdown().split_off(at))
    }

    /// Truncates this `MowStr`, removing all contents.
    ///
    /// While this means the `MowStr` will have a length of zero, it does not
    /// touch its capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.mutdown().clear()
    }

    /// Creates a draining iterator that removes the specified range in the `MowStr`
    /// and yields the removed `chars`.
    ///
    /// Note: The element range is removed even if the iterator is not
    /// consumed until the end.
    ///
    /// # Panics
    ///
    /// Panics if the starting point or end point do not lie on a [`char`]
    /// boundary, or if they're out of bounds.
    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_> {
        self.mutdown().drain(range)
    }

    /// Removes the specified range in the string,
    /// and replaces it with the given string.
    /// The given string doesn't need to be the same length as the range.
    ///
    /// # Panics
    ///
    /// Panics if the starting point or end point do not lie on a [`char`]
    /// boundary, or if they're out of bounds.
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
            Inner::I(v) => Self::from_istr(v.clone()),
            Inner::M(v) => Self::from_string(v.clone().unwrap()),
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
            Inner::I(v) => v.as_ref(),
            Inner::M(v) => v.as_ref().unwrap(),
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
            Inner::I(v) => v.as_ref(),
            Inner::M(v) => v.as_ref().unwrap().as_ref(),
        }
    }
}

impl AsRef<OsStr> for MowStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        match &self.0 {
            Inner::I(v) => v.as_ref(),
            Inner::M(v) => v.as_ref().unwrap().as_ref(),
        }
    }
}

impl AsRef<Path> for MowStr {
    #[inline]
    fn as_ref(&self) -> &Path {
        match &self.0 {
            Inner::I(v) => v.as_ref(),
            Inner::M(v) => v.as_ref().unwrap().as_ref(),
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
        Self::from_boxed(s)
    }
}

impl From<Arc<str>> for MowStr {
    #[inline]
    fn from(s: Arc<str>) -> Self {
        Self::from_arc(s)
    }
}

impl From<Rc<str>> for MowStr {
    #[inline]
    fn from(s: Rc<str>) -> Self {
        Self::from_rc(s)
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
            Inner::I(v) => v.to_string(),
            Inner::M(v) => v.clone().unwrap(),
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
            Inner::I(v) => Self::from(v.deref()),
            Inner::M(v) => Self::from(v.as_deref().unwrap()),
        }
    }
}

impl From<MowStr> for Vec<u8> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            Inner::I(v) => Self::from(v.deref()),
            Inner::M(v) => Self::from(v.as_deref().unwrap()),
        }
    }
}

impl From<MowStr> for Arc<str> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            Inner::I(v) => Self::from(v.clone()),
            Inner::M(v) => Self::from(v.clone().unwrap()),
        }
    }
}

impl From<MowStr> for Rc<str> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            Inner::I(v) => Self::from(v.clone()),
            Inner::M(v) => Self::from(v.clone().unwrap()),
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
            Inner::I(v) => Self::from(v.clone()),
            Inner::M(v) => Self::from(v.clone().unwrap()),
        }
    }
}

impl From<MowStr> for Box<dyn Error + Send + Sync> {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            Inner::I(v) => Self::from(v.clone()),
            Inner::M(v) => Self::from(v.clone().unwrap()),
        }
    }
}

impl From<MowStr> for OsString {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            Inner::I(v) => Self::from(v.deref()),
            Inner::M(v) => Self::from(v.as_ref().unwrap()),
        }
    }
}

impl From<MowStr> for PathBuf {
    #[inline]
    fn from(v: MowStr) -> Self {
        match &v.0 {
            Inner::I(v) => Self::from(v.deref()),
            Inner::M(v) => Self::from(v.as_ref().unwrap()),
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
            Inner::I(v) => v,
            Inner::M(v) => Self::from_string(v.unwrap()),
        }
    }
}

impl PartialEq<str> for MowStr {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

impl PartialEq<&str> for MowStr {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<String> for MowStr {
    fn eq(&self, other: &String) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<OsStr> for MowStr {
    fn eq(&self, other: &OsStr) -> bool {
        self.deref() == other
    }
}

impl PartialEq<&OsStr> for MowStr {
    fn eq(&self, other: &&OsStr) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<OsString> for MowStr {
    fn eq(&self, other: &OsString) -> bool {
        self.deref() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let s = MowStr::new("asd");
        assert_eq!(s, "asd");
    }

    #[test]
    fn test_2() {
        let a = MowStr::new("asd");
        let b = MowStr::new("asd");
        assert_eq!(a, b);
    }

    #[test]
    fn test_3() {
        let a = MowStr::new("asd");
        let b = MowStr::new("123");
        assert_ne!(a, b);
    }

    #[test]
    fn test_mut() {
        let mut a = MowStr::new("asd");
        assert!(a.is_interned());
        a.mutdown();
        assert!(a.is_mutable());
    }

    #[test]
    fn test_mut_2() {
        let mut a = MowStr::new("asd");
        assert!(a.is_interned());
        assert_eq!(a, "asd");
        a.push_str("123");
        assert!(a.is_mutable());
        assert_eq!(a, "asd123");
    }
}
