//! Provides some type conversion utils

use std::{
    ffi::{OsStr, OsString},
    rc::Rc,
    sync::Arc,
};

use crate::{ffi::IOsStr, mow_os_str::MowOsStr, IStr, MowStr};

/// Type annotation
#[doc(hidden)]
pub unsafe trait Interned {}
/// Type annotation
#[doc(hidden)]
pub unsafe trait Muterned {}

/// Convert to Interning String
pub trait Interning {
    type Outern: Interned;

    /// Convert to Interning String
    fn interned(self) -> Self::Outern;
}

/// Convert to Mutable on Write Interning String
pub trait Muterning {
    type Outern: Muterned;

    /// Convert to Mutable on Write Interning String
    fn muterned(self) -> Self::Outern;
}

impl Interning for char {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::from(self)
    }
}

impl Interning for &str {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::new(self)
    }
}

impl Interning for Box<str> {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::from_boxed(self)
    }
}

impl Interning for Arc<str> {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::from_arc(self)
    }
}

impl Interning for Rc<str> {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::from_rc(self)
    }
}

impl Interning for String {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::from_string(self)
    }
}

impl Interning for IStr {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        self
    }
}

impl Interning for MowStr {
    type Outern = MowStr;

    fn interned(mut self) -> Self::Outern {
        self.intern();
        self
    }
}

impl Interning for &OsStr {
    type Outern = IOsStr;

    fn interned(self) -> Self::Outern {
        IOsStr::new(self)
    }
}

impl Interning for OsString {
    type Outern = IOsStr;

    fn interned(self) -> Self::Outern {
        IOsStr::from_os_string(self)
    }
}

impl Interning for Box<OsStr> {
    type Outern = IOsStr;

    fn interned(self) -> Self::Outern {
        IOsStr::from_boxed(self)
    }
}

impl Interning for Arc<OsStr> {
    type Outern = IOsStr;

    fn interned(self) -> Self::Outern {
        IOsStr::from_arc(self)
    }
}

impl Interning for Rc<OsStr> {
    type Outern = IOsStr;

    fn interned(self) -> Self::Outern {
        IOsStr::from_rc(self)
    }
}

impl Interning for IOsStr {
    type Outern = IOsStr;

    fn interned(self) -> Self::Outern {
        self
    }
}

impl Interning for MowOsStr {
    type Outern = MowOsStr;

    fn interned(mut self) -> Self::Outern {
        self.intern();
        self
    }
}

impl Muterning for char {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::new_mut(self)
    }
}

impl Muterning for &str {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::new_mut(self)
    }
}

impl Muterning for Box<str> {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::from_string_mut(self.to_string())
    }
}

impl Muterning for Arc<str> {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::from_string_mut(self.to_string())
    }
}

impl Muterning for Rc<str> {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::from_string_mut(self.to_string())
    }
}

impl Muterning for String {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::from_string_mut(self)
    }
}

impl Muterning for IStr {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::from_string_mut(self.to_string())
    }
}

impl Muterning for MowStr {
    type Outern = MowStr;

    fn muterned(mut self) -> Self::Outern {
        self.to_mut();
        self
    }
}

impl Muterning for &OsStr {
    type Outern = MowOsStr;

    fn muterned(self) -> Self::Outern {
        MowOsStr::new_mut(self)
    }
}

impl Muterning for OsString {
    type Outern = MowOsStr;

    fn muterned(self) -> Self::Outern {
        MowOsStr::from_os_string_mut(self)
    }
}

impl Muterning for Box<OsStr> {
    type Outern = MowOsStr;

    fn muterned(self) -> Self::Outern {
        MowOsStr::from_os_string_mut(self.to_os_string())
    }
}

impl Muterning for Arc<OsStr> {
    type Outern = MowOsStr;

    fn muterned(self) -> Self::Outern {
        MowOsStr::from_os_string_mut(self.to_os_string())
    }
}

impl Muterning for Rc<OsStr> {
    type Outern = MowOsStr;

    fn muterned(self) -> Self::Outern {
        MowOsStr::from_os_string_mut(self.to_os_string())
    }
}

impl Muterning for IOsStr {
    type Outern = MowOsStr;

    fn muterned(self) -> Self::Outern {
        MowOsStr::from_i_os_str(self)
    }
}

impl Muterning for MowOsStr {
    type Outern = MowOsStr;

    fn muterned(self) -> Self::Outern {
        self
    }
}
