use {
    super::{json_rpc, transport::*},
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

    pub async fn handle_connection(&self) -> crate::Result<()> {
        let mut sender = self.sender.lock().await;
        if sender.is_some() {
            return Ok(());
        }

        let connection = TcpStream::connect(self.service_addr).await?;
        let (read_half, write_half) = io::split(connection);
        self.transporter.spawn_reader(read_half);
        *sender = Some(self.transporter.spawn_writer(write_half));

        Ok(())
    }

    pub async fn call(&self, mut request: json_rpc::Request) -> crate::Result<json_rpc::Response> {
        self.handle_connection().await?;

        request.id = Some(json_rpc::Id::Num(rand::random::<u32>()));

        let (tx, rx) = oneshot::channel::<json_rpc::Response>();

        let data_to_send = serde_json::to_vec(&request).unwrap();
        let mut res = Vec::from(data_to_send.len().to_le_bytes());
        res.extend(data_to_send);

        self.transporter
            .clone()
            .add_receiver(request.id.unwrap(), tx);

        match self.sender.lock().await.as_mut() {
            Some(sender) => sender.send(res).await?,
            None => return Err(String::from("io error").into()),
        }

        Ok(rx.await?)
    }

    pub async fn notify(&self, request: json_rpc::Request) -> crate::Result<()> {
        self.handle_connection().await?;

        let data_to_send = serde_json::to_vec(&request).unwrap();
        let mut res = Vec::from(data_to_send.len().to_le_bytes());
        res.extend(data_to_send);

        match self.sender.lock().await.as_mut() {
            Some(sender) => sender.send(res).await?,
            None => return Err(String::from("io error").into()),
        }

        Ok(())
    }
}
