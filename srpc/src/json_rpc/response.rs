use {
    super::*,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::convert::TryFrom,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub jsonrpc: Version,
    pub result: Option<Value>,
    pub error: Option<Error>,
    pub id: Id,
}

impl Response {
    pub fn from_error_data(kind: ErrorKind, data: Option<Value>, id: Id) -> Self {
        Response {
            jsonrpc: Version::V2,
            result: None,
            error: Some(Error::new(kind, data)),
            id,
        }
    }

    pub fn from_result(result: Value, id: Id) -> Self {
        Response {
            jsonrpc: Version::V2,
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn from_error(error: Error, id: Id) -> Self {
        Response {
            jsonrpc: Version::V2,
            result: None,
            error: Some(error),
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
