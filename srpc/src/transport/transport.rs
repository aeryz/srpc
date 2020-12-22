use {
    super::{json_rpc, Reader},
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
                Some(Ok(data)) => {
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

    async fn writer(mut receiver: mpsc::Receiver<Vec<u8>>, mut writer: WriteHalf<TcpStream>) {
        while let Some(data) = receiver.recv().await {
            let mut start_pos = 0;
            let data_len = data.len();
            loop {
                match writer.write(&data[start_pos..data_len]).await {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }
                        start_pos += n;
                    }
                    Err(e) => {
                        error!("IO error occured while writing {}", e);
                        break;
                    }
                }
            }
        }
    }
}
