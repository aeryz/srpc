pub mod client;
pub mod json_rpc;
pub mod server;
pub mod transport;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use srpc_macros::*;
