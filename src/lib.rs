//! ***Global string intern pool***
//!
//! Internal use of [DashMap](https://crates.io/crates/dashmap) for concurrent support
//!
//! # Examples
//! - [`IStr`](struct.IStr.html)
//! ```
//! use pstr::IStr;
//! let s = IStr::new("hello world");
//! ```
//! - [`MowStr`](struct.MowStr.html)
//! ```
//! use pstr::MowStr;
//! let mut s = MowStr::new("hello");
//! assert!(s.is_interned());
//!
//! s.push_str(" ");
//! assert!(s.is_mutable());
//!
//! s.mutdown().push_str("world");
//! assert_eq!(s, "hello world");
//!
//! s.intern();
//! assert!(s.is_interned());
//! ```

pub mod intern;
mod istr;
mod mow_str;
pub mod pool;
mod prc;
pub use intern::{Interning, Muterning};
pub use istr::*;
pub use mow_str::*;
