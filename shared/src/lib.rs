pub mod calendar;
pub mod commands;
pub mod display_interface;
pub mod domain;
pub mod error;
pub mod events;
pub mod modules;
pub mod persistence;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
