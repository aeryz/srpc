use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    sync::mpsc::Receiver,
};

pub struct TransportData<T>
where
    T: AsyncWrite + Unpin,
{
    pub stream: T,
    pub data: Vec<u8>,
}

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
    pub async fn listen(&mut self) {
        while let Some(mut data) = self.receiver.recv().await {
            data.stream.write_all(&data.data).await.unwrap();
        }
    }
}
