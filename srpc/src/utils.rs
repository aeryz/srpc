use crate::json_rpc::*;

use super::Result;

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

/// Reads data from a reader until error occurs, read size is 0 or "\n\r" is seen at the end
/// of the read data.
pub async fn read_frame<T: AsyncReadExt + Unpin>(stream: Arc<Mutex<T>>) -> Result<Vec<u8>> {
    let mut total_data = Vec::new();
    loop {
        let mut data = [0 as u8; 1024];
        match stream.lock().await.read(&mut data).await {
            Ok(0) => break,
            Ok(n) => {
                total_data.extend_from_slice(&data[0..n]);
            }
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
    println!(
        "read data is {}",
        String::from_utf8(total_data.clone()).unwrap()
    );
    Ok(total_data)
}

pub async fn read_frame2<T: AsyncReadExt + Unpin>(stream: &mut T) -> Result<Vec<u8>> {
    let mut total_data = Vec::new();
    loop {
        let mut data = [0 as u8; 1024];
        match stream.read(&mut data).await {
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

pub async fn write<W: AsyncWrite + Unpin>(mut receiver: Receiver<super::Data<W>>) {
    while let Some(data) = receiver.recv().await {
        let mut start_pos = 0;
        let data_len = data.res.len();
        while let Ok(n) = data
            .stream
            .lock()
            .await
            .write(&data.res[start_pos..data_len])
            .await
        {
            if n == 0 {
                break;
            }
            start_pos += n;
        }
    }
}

pub async fn send_error_response<T: AsyncWrite + Unpin>(
    sender: Sender<super::Data<T>>,
    stream: Arc<Mutex<T>>,
    error_kind: ErrorKind,
    error_data: Option<serde_json::Value>,
    rpc_id: Id,
) {
    println!("Sending err");
    let mut res = Vec::new();
    let err = serde_json::to_vec(&Response::new_error(error_kind, error_data, rpc_id)).unwrap();
    res.extend(&err.len().to_le_bytes());
    res.extend(err);
    let _ = sender
        .send(super::Data {
            stream: stream.clone(),
            res,
        })
        .await;
    println!("Err sent");
}

pub async fn send_result_response<T: AsyncWrite + Unpin>(
    sender: Sender<super::Data<T>>,
    stream: Arc<Mutex<T>>,
    result: serde_json::Value,
    rpc_id: Id,
) {
    println!("Sending res");
    let mut res = Vec::new();
    let data = serde_json::to_vec(&Response::new_result(result, rpc_id)).unwrap();
    res.extend(&data.len().to_le_bytes());
    res.extend(data);
    let _ = sender
        .send(super::Data {
            stream: stream.clone(),
            res,
        })
        .await;
}
