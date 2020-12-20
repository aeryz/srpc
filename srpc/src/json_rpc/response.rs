use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub jsonrpc: Version,
    pub result: Option<Value>,
    pub error: Option<Error>,
    pub id: Id,
}

impl Response {
    pub fn new_error(kind: ErrorKind, data: Option<Value>, id: Id) -> Self {
        Response {
            jsonrpc: Version::V2,
            result: None,
            error: Some(Error::new(kind, data)),
            id,
        }
    }

    pub fn new_result(result: Value, id: Id) -> Self {
        Response {
            jsonrpc: Version::V2,
            result: Some(result),
            error: None,
            id,
        }
    }
}

impl TryFrom<&[u8]> for Response {
    type Error = serde_json::Error;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice::<Response>(data)
    }
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}
