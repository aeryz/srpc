/// The async RPC client.
///
/// The client connects a server and unless a connection error occurs, it does not
/// drop the connection. Timeouts are not supported yet.
///
/// # Example
/// Check out the examples directory to see examples.
///
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
    // Sends data to writer
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

    /// Provides a persistent connection.
    pub async fn handle_connection(&self) -> crate::Result<()> {
        let mut sender = self.sender.lock().await;
        // Do nothing if there is already an open connection
        if sender.is_some() {
            return Ok(());
        }

        let connection = TcpStream::connect(self.service_addr).await?;
        let (read_half, write_half) = io::split(connection);
        self.transporter.spawn_reader(read_half);
        *sender = Some(self.transporter.spawn_writer(write_half)); // Save the sender coming from the writer

        Ok(())
    }

    pub fn create_data(&self, request: &json_rpc::Request) -> crate::Result<Vec<u8>> {
        let data_to_send = serde_json::to_vec(&request).unwrap();
        // To support 32 bit machines easily
        if data_to_send.len() > std::u32::MAX as usize {
            Err(format!("max data size ({}) is exceeded.", std::u32::MAX).into())
        } else {
            Ok(data_to_send)
        }
    }

    /// Makes an rpc call and waits for the response 
    pub async fn call(&self, mut request: json_rpc::Request) -> crate::Result<json_rpc::Response> {
        self.handle_connection().await?;

        request.id = Some(json_rpc::Id::Num(rand::random::<u32>()));

        let (tx, rx) = oneshot::channel::<json_rpc::Response>();

        let req = self.create_data(&request)?;

        // Register to the receivers to receive the correct response
        self.transporter
            .clone()
            .add_receiver(request.id.unwrap(), tx);

        match self.sender.lock().await.as_mut() {
            Some(sender) => sender.send(req).await?,
            None => return Err(String::from("io error").into()),
        }
        Ok(rx.await?)
    }

    /// Makes an rpc notification call and DOES NOT wait for the response
    pub async fn notify(&self, request: json_rpc::Request) -> crate::Result<()> {
        self.handle_connection().await?;

        let res = self.create_data(&request)?;

        match self.sender.lock().await.as_mut() {
            Some(sender) => sender.send(res).await?,
            None => return Err(String::from("io error").into()),
        }
        Ok(())
    }
}
