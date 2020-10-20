use crate::{IStr, MowStr};

pub unsafe trait Interned {}
pub unsafe trait Muterned {}

pub trait Interning {
    type Outern: Interned;

    fn interned(self) -> Self::Outern;
}

pub trait Muterning {
    type Outern: Muterned;

    fn muterned(self) -> Self::Outern;
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

impl Muterning for &str {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::new_mut(self)
    }
}

impl Muterning for Box<str> {
    type Outern = MowStr;

    fn muterned(self) -> Self::Outern {
        MowStr::from_boxed_str_mut(self)
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
        MowStr::from_istr_mut(self)
    }
}

impl Muterning for MowStr {
    type Outern = MowStr;

    fn muterned(mut self) -> Self::Outern {
        self.to_mut();
        self
    }
}
