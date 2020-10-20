use crate::{IStr, MowStr};

pub unsafe trait Interned {}

pub trait Interning {
    type Outern: Interned;

    fn interned(self) -> Self::Outern;
}

impl Interning for &str {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::from(self)
    }
}

impl Interning for Box<str> {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::from(self)
    }
}

impl Interning for String {
    type Outern = IStr;

    fn interned(self) -> Self::Outern {
        IStr::from(self)
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
