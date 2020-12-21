use {
    super::{json_rpc::*, transport::*},
    std::{net::SocketAddr, sync::Arc},
    tokio::{
        io,
        net::TcpStream,
        sync::{mpsc, oneshot, Mutex},
    },
};

pub struct Client {
    sender: Arc<Mutex<Option<mpsc::Sender<Vec<u8>>>>>,
    service_addr: SocketAddr,
    transporter: Arc<Transport>,
}

impl Client {
    pub fn new(service_addr: SocketAddr, transporter: Arc<Transport>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(None)),
            service_addr,
            transporter,
        }
    }

    pub async fn handle_connection(&self) {
        let mut sender = self.sender.lock().await;
        if sender.is_some() {
            return;
        }

        let connection = TcpStream::connect(self.service_addr).await.unwrap();
        let (read_half, write_half) = io::split(connection);
        self.transporter.spawn_reader(read_half);
        *sender = Some(self.transporter.spawn_writer(write_half));
    }

    pub async fn call(&self, mut request: Request) -> Response {
        self.handle_connection().await;
        request.id = Some(Id::Num(rand::random::<u32>()));
        let (tx, rx) = oneshot::channel::<Response>();

        let mut res = Vec::new();
        let data_to_send = serde_json::to_vec(&request).unwrap();
        res.extend(&data_to_send.len().to_le_bytes());
        res.extend(data_to_send);

        self.transporter
            .clone()
            .add_receiver(request.id.unwrap(), tx);

        let _ = self.sender.lock().await.as_mut().unwrap().send(res).await;
        rx.await.unwrap()
    }
}
