use {
    super::{
        json_rpc::{self, Response},
        Reader,
    },
    futures::StreamExt,
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
            println!("Will write");
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
