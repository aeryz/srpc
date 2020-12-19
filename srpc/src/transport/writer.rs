use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    sync::mpsc::{self, Receiver, Sender},
};

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Data<T>
where
    T: AsyncWrite + Unpin,
{
    pub stream: Arc<Mutex<T>>,
    pub data: Vec<u8>,
}

impl<T> Data<T>
where
    T: AsyncWrite + Unpin,
{
    pub fn new(stream: Arc<Mutex<T>>, data: Vec<u8>) -> Self {
        Self { stream, data }
    }
}

#[derive(Debug)]
pub struct Writer<T>
where
    T: AsyncWrite + Unpin,
{
    pub receiver: Receiver<Data<T>>,
}

#[derive(Debug)]
pub struct PersistantWriter<W>
where
    W: AsyncWrite + Unpin,
{
    pub receiver: Receiver<Vec<u8>>,
    pub writer: W,
}

impl<W> PersistantWriter<W>
where
    W: AsyncWrite + Unpin,
{
    pub fn new(writer: W) -> (Self, Sender<Vec<u8>>) {
        let (sender, receiver) = mpsc::channel(32);
        (Self { receiver, writer }, sender)
    }

    pub async fn write_incoming(&mut self) {
        while let Some(data) = self.receiver.recv().await {
            let mut start_pos = 0;
            let data_len = data.len();
            while let Ok(n) = self.writer.write(&data[start_pos..data_len]).await {
                if n == 0 {
                    break;
                }
                start_pos += n;
            }
        }
    }
}

impl<T> Writer<T>
where
    T: AsyncWrite + Unpin,
{
    pub fn new(receiver: Receiver<Data<T>>) -> Self {
        Self { receiver }
    }

    pub async fn write_incoming(&mut self) {
        while let Some(data) = self.receiver.recv().await {
            let mut start_pos = 0;
            let data_len = data.data.len();
            while let Ok(n) = data
                .stream
                .lock()
                .await
                .write(&data.data[start_pos..data_len])
                .await
            {
                if n == 0 {
                    break;
                }
                start_pos += n;
            }
        }
    }
}
