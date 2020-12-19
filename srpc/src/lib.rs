pub mod client_;
pub mod json_rpc;
pub mod server;
pub mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use client_::*;
pub use srpc_macros::*;

use std::sync::Arc;
use tokio::io::AsyncWrite;
use tokio::sync::Mutex;
pub struct Data<W: AsyncWrite + Unpin> {
    stream: Arc<Mutex<W>>,
    res: Vec<u8>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
