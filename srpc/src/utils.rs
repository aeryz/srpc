use crate::{json_rpc::*, transport::TransportData};

use super::Result;

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWrite};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

/// Reads data from a reader until error occurs, read size is 0 or "\n\r" is seen at the end
/// of the read data.
pub async fn read_frame<T: AsyncReadExt + Unpin>(stream: Arc<Mutex<T>>) -> Result<Vec<u8>> {
    let mut total_data = Vec::new();
    loop {
        let mut data = [0 as u8; 1024];
        match stream.lock().await.read(&mut data).await {
            Ok(0) => break,
            Ok(n) => total_data.extend_from_slice(&data[0..n]),
            Err(e) => return Err(Box::new(e)),
        }
        if total_data.len() > 1
            && total_data[total_data.len() - 1] == b'\n'
            && total_data[total_data.len() - 2] == b'\r'
        {
            total_data.resize(total_data.len() - 2, 0);
            break;
        }
    }
    Ok(total_data)
}

pub async fn send_error_response<T: AsyncWrite + Unpin>(
    sender: Sender<TransportData<T>>,
    stream: Arc<Mutex<T>>,
    error_kind: ErrorKind,
    error_data: Option<serde_json::Value>,
    rpc_id: Id,
) {
    let err = Response::new_error(error_kind, error_data, rpc_id);
    let _ = sender
        .send(TransportData::new(
            stream.clone(),
            serde_json::to_vec(&err).unwrap(),
        ))
        .await;
}

pub async fn send_result_response<T: AsyncWrite + Unpin>(
    sender: Sender<TransportData<T>>,
    stream: Arc<Mutex<T>>,
    result: serde_json::Value,
    rpc_id: Id,
) {
    let err = Response::new_result(result, rpc_id);
    let _ = sender
        .send(TransportData::new(
            stream.clone(),
            serde_json::to_vec(&err).unwrap(),
        ))
        .await;
}
