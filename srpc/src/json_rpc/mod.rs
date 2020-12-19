mod errors;
mod request;
mod response;

pub use errors::*;
pub use request::*;
pub use response::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Id {
    Str(String),
    Num(u32),
}

unsafe impl Send for Id {}
