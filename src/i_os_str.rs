use std::{
    borrow::Borrow, convert::identity, convert::Infallible, ffi::OsStr, ffi::OsString, hash,
    hash::Hash, ops::Deref, path::Path, path::PathBuf, rc::Rc, str::FromStr, sync::Arc,
};

use crate::{
    intern::Interned,
    mow_os_str::MowOsStr,
    pool::{Intern, OS_STR_POOL},
};

/// Immutable Interning OsString
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IOsStr(Intern<OsStr>);

impl IOsStr {
    /// Create a `IOsStr` from str slice  
    ///
    /// # Example
    /// ```
    /// # use pstr::ffi::IOsStr;
    /// let s = IOsStr::new("hello world");
    /// ```
    #[inline]
    pub fn new(s: impl AsRef<OsStr>) -> Self {
        Self(OS_STR_POOL.intern(s.as_ref(), Arc::from))
    }

    /// Create a `IOsStr` from `OsString`  
    #[inline]
    pub fn from_os_string(s: OsString) -> Self {
        Self(OS_STR_POOL.intern(s, Arc::from))
    }

    /// Create a `IOsStr` from `Box<OsStr>`  
    #[inline]
    pub fn from_boxed(s: Box<OsStr>) -> Self {
        Self(OS_STR_POOL.intern(s, Arc::from))
    }

    /// Create a `IOsStr` from `Arc<OsStr>`  
    #[inline]
    pub fn from_arc(s: Arc<OsStr>) -> Self {
        Self(OS_STR_POOL.intern(s, identity))
    }

    /// Create a `IOsStr` from `Rc<OsStr>`  
    #[inline]
    pub fn from_rc(s: Rc<OsStr>) -> Self {
        Self(OS_STR_POOL.intern(s, |s| Arc::from(s.to_os_string())))
    }

    /// Create a `IOsStr` from `MowOsStr`
    #[inline]
    pub fn from_mow(s: MowOsStr) -> Self {
        s.into()
    }

    /// Create a `IOsStr` from custom fn  
    #[inline]
    pub fn from_to_arc<S: AsRef<OsStr>>(s: S, to_arc: impl FnOnce(S) -> Arc<OsStr>) -> Self {
        Self(OS_STR_POOL.intern(s, to_arc))
    }
}

impl IOsStr {
    /// Converts to an `OsStr` slice.
    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        self.deref()
    }

    /// Converts to an `Box<OsStr>`.
    #[inline]
    pub fn into_boxed_os_str(&self) -> Box<OsStr> {
        self.deref().into()
    }

    /// Convert to `MowStr`
    #[inline]
    pub fn into_mut(&self) -> MowOsStr {
        MowOsStr::from(self.clone())
    }
}

unsafe impl Interned for IOsStr {}

impl Deref for IOsStr {
    type Target = OsStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<OsStr> for IOsStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.0.get()
    }
}

impl AsRef<Path> for IOsStr {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.deref().as_ref()
    }
}

impl Hash for IOsStr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl Borrow<OsStr> for IOsStr {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self.deref()
    }
}

impl From<&'_ String> for IOsStr {
    #[inline]
    fn from(s: &'_ String) -> Self {
        Self::new(s)
    }
}

impl From<&'_ str> for IOsStr {
    #[inline]
    fn from(s: &'_ str) -> Self {
        Self::new(s)
    }
}

impl From<&'_ mut str> for IOsStr {
    #[inline]
    fn from(s: &'_ mut str) -> Self {
        Self::new(s)
    }
}

impl From<char> for IOsStr {
    #[inline]
    fn from(c: char) -> Self {
        let mut tmp = [0; 4];
        Self::new(c.encode_utf8(&mut tmp))
    }
}

impl From<Box<OsStr>> for IOsStr {
    #[inline]
    fn from(s: Box<OsStr>) -> Self {
        Self::from_boxed(s)
    }
}

impl From<Arc<OsStr>> for IOsStr {
    #[inline]
    fn from(s: Arc<OsStr>) -> Self {
        Self::from_arc(s)
    }
}

impl From<Rc<OsStr>> for IOsStr {
    #[inline]
    fn from(s: Rc<OsStr>) -> Self {
        Self::from_rc(s)
    }
}

impl From<PathBuf> for IOsStr {
    #[inline]
    fn from(s: PathBuf) -> Self {
        Self::from_os_string(s.into())
    }
}

impl From<OsString> for IOsStr {
    #[inline]
    fn from(s: OsString) -> Self {
        Self::from_os_string(s)
    }
}

impl From<&'_ OsStr> for IOsStr {
    #[inline]
    fn from(s: &OsStr) -> Self {
        Self::new(s)
    }
}

impl From<&'_ OsString> for IOsStr {
    #[inline]
    fn from(s: &OsString) -> Self {
        Self::new(s)
    }
}

impl From<String> for IOsStr {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_os_string(s.into())
    }
}

impl FromStr for IOsStr {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OsString::from_str(s).map(Self::from_os_string)
    }
}

impl From<IOsStr> for Box<OsStr> {
    #[inline]
    fn from(v: IOsStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IOsStr> for Arc<OsStr> {
    #[inline]
    fn from(v: IOsStr) -> Self {
        v.0.into()
    }
}

impl From<IOsStr> for Rc<OsStr> {
    #[inline]
    fn from(v: IOsStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IOsStr> for OsString {
    #[inline]
    fn from(v: IOsStr) -> Self {
        Self::from(v.deref())
    }
}

impl From<IOsStr> for PathBuf {
    #[inline]
    fn from(v: IOsStr) -> Self {
        Self::from(v.deref())
    }
}

impl PartialEq<OsStr> for IOsStr {
    fn eq(&self, other: &OsStr) -> bool {
        self.deref() == other
    }
}

impl PartialEq<&OsStr> for IOsStr {
    fn eq(&self, other: &&OsStr) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<OsString> for IOsStr {
    fn eq(&self, other: &OsString) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<str> for IOsStr {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

impl PartialEq<&str> for IOsStr {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<String> for IOsStr {
    fn eq(&self, other: &String) -> bool {
        self.deref() == other.as_str()
    }
}
