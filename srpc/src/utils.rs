use super::protocol::*;
use super::Result;

pub fn throw_if_error(status_code: StatusCodeType) -> Result<()> {
    if status_code == StatusCode::SUCCESS {
        Ok(())
    } else {
        Err(Box::new(SrpcError(status_code)))
    }
}
