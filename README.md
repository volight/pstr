***Global string intern pool***  

![Rust](https://github.com/volight/pstr/workflows/Rust/badge.svg)
[![version](https://img.shields.io/crates/v/pstr)](https://crates.io/crates/pstr)
[![documentation](https://docs.rs/pstr/badge.svg)](https://docs.rs/pstr)

Internal use of [DashMap](https://crates.io/crates/dashmap) for concurrent support

# Examples
- [`IStr`](https://docs.rs/pstr/0.4.0/pstr/struct.IStr.html)
```rust
use pstr::IStr;
let s = IStr::new("hello world");
```
- [`MowStr`](https://docs.rs/pstr/0.4.0/pstr/struct.MowStr.html)
```rust
use pstr::MowStr;
let mut s = MowStr::new("hello");
assert!(s.is_interned());

s.push_str(" ");
assert!(s.is_mutable());

s.mutdown().push_str("world");
assert_eq!(s, "hello world");

s.intern();
assert!(s.is_interned());
```
