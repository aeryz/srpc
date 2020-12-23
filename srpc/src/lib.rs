/// # Flaws
/// No mechanism exists to limit the read or written data size. Server's can easily be DDOS'd.
/// A limit should be defined.
///
/// If a request or response exceeds the data size limit, it should be framed into multiple
/// packages.
///
/// Currently using usize as the HEADER_LEN. But usize varies between x64 and x86. There should
/// be a workaround for x86 or 'u32' should be used.
pub mod client;
pub mod json_rpc;
pub mod server;
pub mod transport;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use srpc_macros::*;
