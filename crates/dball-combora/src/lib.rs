pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub mod checker;
pub mod dball;
pub mod generator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
