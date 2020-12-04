use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub type StatusCodeType = u16;

pub mod StatusCode {
    use super::StatusCodeType;
    pub static SUCCESS: StatusCodeType = 200;
    pub static NOT_FOUND: StatusCodeType = 404;
}

#[derive(Serialize, Deserialize)]
pub struct SrpcRequest<'a, T> {
    pub route: &'a str,
    pub method_name: &'a str,
    pub data: T,
}

impl<'a, T> SrpcRequest<'a, T> {
    pub fn new(route: &'a str, method_name: &'a str, data: T) -> Self {
        Self {
            route,
            method_name,
            data,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SrpcResponse<T> {
    // TODO: Error message
    pub status_code: u16,
    pub data: T,
}

impl<T> SrpcResponse<T> {
    pub fn new(status_code: StatusCodeType, data: T) -> Self {
        Self { status_code, data }
    }
}

#[derive(Debug)]
pub struct SrpcError(pub StatusCodeType);

impl Error for SrpcError {}

impl fmt::Display for SrpcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rpc error occured. Error code: {}", self.0)
    }
}
