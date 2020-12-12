use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcId {
    Str(String),
    Number(i32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub jsonrpc: String,
    pub route: String,
    pub method: String,
    pub params: Value,
    pub id: Option<RpcId>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<RpcError>,
    pub id: RpcId,
}

impl TryFrom<&[u8]> for Request {
    type Error = RpcError;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice::<Request>(data).map_err(|e| {
            RpcError::new(
                RpcErrorCode::INVALID_REQUEST,
                "Invalid Request".to_owned(),
                Some(Value::String(e.to_string())),
            )
        })
    }
}

impl TryFrom<&[u8]> for Response {
    type Error = serde_json::Error;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice::<Response>(data)
    }
}

impl RpcError {
    pub fn new(code: i32, message: String, data: Option<Value>) -> Self {
        RpcError {
            code,
            message,
            data,
        }
    }
}

#[allow(non_snake_case)]
pub mod RpcErrorCode {
    pub static PARSE_ERROR: i32 = -32700;
    pub static INVALID_REQUEST: i32 = -32600;
    pub static METHOD_NOT_FOUND: i32 = -32601;
    pub static INVALID_PARAMS: i32 = -32602;
    pub static INTERNAL_ERRORS: i32 = -32603;
    pub static SERVER_ERRORS: i32 = -32000;
}
