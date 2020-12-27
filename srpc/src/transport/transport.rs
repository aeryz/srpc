use {
    super::{codec, json_rpc, Reader},
    futures::StreamExt,
    log::{error, info, warn},
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
    tokio::{
        io::{AsyncWriteExt, ReadHalf, WriteHalf},
        net::TcpStream,
        sync::{mpsc, oneshot},
    },
};

type Receivers = HashMap<json_rpc::Id, oneshot::Sender<json_rpc::Response>>;

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

    pub fn add_receiver(
        self: Arc<Self>,
        id: json_rpc::Id,
        sender: oneshot::Sender<json_rpc::Response>,
    ) {
        log::debug!("Receiver length: {}", self.receivers.lock().unwrap().len());
        self.receivers.lock().unwrap().insert(id, sender);
    }

    async fn reader(receivers: Arc<Mutex<Receivers>>, reader: ReadHalf<TcpStream>) {
        let mut reader: Reader<json_rpc::Response, _> = Reader::new(reader);
        loop {
            let next = reader.next().await;
            match next {
                Some(Ok(codec::Type::Single(data))) => {
                    let sender = {
                        let mut receivers = receivers.lock().unwrap();
                        receivers.remove(&data.id)
                    };
                    if let Some(sender) = sender {
                        sender.send(data).unwrap();
                    } else {
                        warn!("Response came with an unexpected identifier. Ignoring.");
                    }
                }
                Some(Ok(codec::Type::Batched(_))) => {
                    panic!("Client does not support batched requests yet.");
                }
                Some(Err(e)) => {
                    error!("IO error occured during reading: {}", e);
                    break;
                }
                None => {
                    info!("Hit the EOF during reading.");
                    break;
                }
            }
        }
    }

    async fn write_buf(
        writer: &mut WriteHalf<TcpStream>,
        buffer: &[u8],
    ) -> Result<(), std::io::Error> {
        let mut start_pos = 0;
        loop {
            let n = writer.write(&buffer[start_pos..]).await?;
            if n == 0 {
                return Ok(());
            }
            start_pos += n;
        }
    }

    async fn writer(mut receiver: mpsc::Receiver<Vec<u8>>, mut writer: WriteHalf<TcpStream>) {
        while let Some(data) = receiver.recv().await {
            if let Err(e) =
                Transport::write_buf(&mut writer, &(data.len() as u32).to_le_bytes()).await
            {
                log::error!("error occured during writing data {}", e);
            }
            if let Err(e) = Transport::write_buf(&mut writer, &data[..]).await {
                log::error!("error occured during writing data {}", e);
            }
        }
    }
}
