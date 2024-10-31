pub mod calendar;
pub mod commands;
pub mod contract;
pub mod display_interface;
pub mod domain;
pub mod error;
pub mod events;
pub mod fasttrack;
pub mod message_bus;
pub mod modules;
pub mod persistence;
pub mod reminders;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

fn main() {}
