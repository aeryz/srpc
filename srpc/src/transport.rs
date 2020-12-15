use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    sync::mpsc::Receiver,
};

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct TransportData<T>
where
    T: AsyncWrite + Unpin,
{
    pub stream: Arc<Mutex<T>>,
    pub data: Vec<u8>,
}

impl<T> TransportData<T>
where
    T: AsyncWrite + Unpin,
{
    pub fn new(stream: Arc<Mutex<T>>, data: Vec<u8>) -> Self {
        Self { stream, data }
    }
}

#[derive(Debug)]
pub struct Transport<T>
where
    T: AsyncWrite + Unpin,
{
    pub receiver: Receiver<TransportData<T>>,
}

impl<T> Transport<T>
where
    T: AsyncWrite + Unpin,
{
    pub fn new(receiver: Receiver<TransportData<T>>) -> Self {
        Transport { receiver }
    }

    pub async fn listen(&mut self) {
        while let Some(data) = self.receiver.recv().await {
            println!("Came to transport");
            let mut start_pos = 0;
            let mut data_len = 5;
            while let Ok(n) = data
                .stream
                .lock()
                .await
                .write(&data.data[start_pos..data_len])
                .await
            {
                println!("Wrote {} bytes", n);
                if n == 0 {
                    break;
                }
                data_len += 5;
                if data_len > data.data.len() {
                    data_len = data.data.len();
                }
                start_pos += n;
            }
        }
    }
}
