pub mod client;
pub mod protocol;
pub mod server;
pub mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub use srpc_macros::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
