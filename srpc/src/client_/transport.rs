use super::json_rpc::{self, Request};
use super::simple_codec::*;
use super::Response;
use super::Result;
use futures::stream::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::io::ReadBuf;
use tokio::io::WriteHalf;
use tokio::io::{AsyncRead, AsyncWriteExt, ReadHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};

pub struct Data {
    writer: WriteHalf<TcpStream>,
    data: Vec<u8>,
}

type Receivers = HashMap<json_rpc::Id, oneshot::Sender<Response>>;

pub struct Transport {
    receivers: Arc<Mutex<Receivers>>,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            receivers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn spawn_reader(self: &Arc<Self>, reader: ReadHalf<TcpStream>) {
        let receivers = self.receivers.clone();
        tokio::spawn(Transport::reader(receivers, reader));
    }

    pub fn spawn_writer(self: &Arc<Self>, writer: WriteHalf<TcpStream>) -> mpsc::Sender<Vec<u8>> {
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(Transport::writer(rx, writer));
        tx
    }

    pub fn add_receiver(self: Arc<Self>, id: json_rpc::Id, sender: oneshot::Sender<Response>) {
        self.receivers.lock().unwrap().insert(id, sender);
    }

    async fn reader(receivers: Arc<Mutex<Receivers>>, reader: ReadHalf<TcpStream>) {
        let mut reader: Reader<Response, _> = Reader::new(reader);
        loop {
            let next = reader.next().await;
            match next {
                Some(Ok(data)) => {
                    let sender = {
                        let mut receivers = receivers.lock().unwrap();
                        receivers.remove(&data.id).unwrap()
                    };
                    sender.send(data).unwrap();
                }
                _ => break,
            }
        }
    }

    async fn writer(mut receiver: mpsc::Receiver<Vec<u8>>, mut writer: WriteHalf<TcpStream>) {
        while let Some(data) = receiver.recv().await {
            let mut start_pos = 0;
            let data_len = data.len();
            while let Ok(n) = writer.write(&data[start_pos..data_len]).await {
                if n == 0 {
                    break;
                }
                start_pos += n;
            }
        }
    }
}

pub struct Reader<D, R> {
    codec: SimpleCodec<D>,
    reader: Pin<Box<R>>,
}

impl<D, R> Reader<D, R>
where
    D: DeserializeOwned,
    R: AsyncRead + Unpin,
{
    pub fn new(reader: R) -> Self {
        Self {
            codec: SimpleCodec::new(),
            reader: Box::pin(reader),
        }
    }
}

impl<D, R> Stream for Reader<D, R>
where
    D: DeserializeOwned,
    R: AsyncRead + Unpin,
{
    type Item = Result<D>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let self_ref = unsafe { &mut self.get_unchecked_mut() };
        if let Some(Ok(data)) = self_ref.codec.drain() {
            return Poll::Ready(Some(Ok(data)));
        }
        let mut buffer = [0 as u8; 1024];
        let mut buf = ReadBuf::new(&mut buffer);
        loop {
            match self_ref.reader.as_mut().poll_read(cx, &mut buf) {
                Poll::Ready(Ok(_)) => {
                    if buf.filled().is_empty() {
                        println!("Poll: Ok() None");
                        return Poll::Ready(None);
                    }
                    println!("Poll: Ok()");
                    self_ref.codec.extend(buf.filled());
                    if let Some(Ok(data)) = self_ref.codec.drain() {
                        return Poll::Ready(Some(Ok(data)));
                    }
                }
                Poll::Ready(Err(e)) => {
                    println!("Poll: Err(e)");
                    return Poll::Ready(Some(Err(e.into())));
                }
                Poll::Pending => {
                    println!("Poll: Pending");
                    return Poll::Pending;
                }
            }
        }
    }
}
