mod istr;
mod mow_str;
mod pool;
pub use istr::*;
pub use mow_str::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
