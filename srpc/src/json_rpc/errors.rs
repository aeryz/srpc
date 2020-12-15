use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub enum ErrorKind {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    ServerError(u32),
}

// TODO: Custom (de)serialization is needed.
// Error should be serialized to:
// {
//    "code": -32700,
//    "message": "Parse error",
//    "data": { .. }
// }
#[derive(Debug, Deserialize, Serialize)]
pub struct Error {
    pub kind: ErrorKind,
    pub data: Option<Value>,
}

impl Error {
    pub fn new(kind: ErrorKind, data: Option<Value>) -> Self {
        Self { kind, data }
    }
}

impl ErrorKind {
    pub fn code(&self) -> i32 {
        match *self {
            ErrorKind::ParseError => -32700,
            ErrorKind::InvalidRequest => -32700,
            ErrorKind::MethodNotFound => -32700,
            ErrorKind::InvalidParams => -32700,
            ErrorKind::InternalError => -32700,
            ErrorKind::ServerError(n) => -32700 - n as i32,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            ErrorKind::ParseError => "Parse error",
            ErrorKind::InvalidRequest => "Invalid Request",
            ErrorKind::MethodNotFound => "Method not found",
            ErrorKind::InvalidParams => "Invalid params",
            ErrorKind::InternalError => "Internal error",
            ErrorKind::ServerError(_) => "Server error",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rpc error!")
    }
}

impl std::error::Error for Error {}
