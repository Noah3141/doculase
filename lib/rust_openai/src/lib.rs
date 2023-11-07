// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

pub mod models;
pub mod client;
pub mod batteries;

pub mod constants;

pub use client::OpenAIAccount;
pub use batteries::Battery;
pub use models::GptModel;
pub use models::Query;