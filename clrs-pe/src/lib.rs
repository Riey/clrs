#[macro_use]
mod utils;
pub mod cil;
pub mod pe;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
