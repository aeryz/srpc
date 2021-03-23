//! The async RPC client.
//!
//! The [Client]() connects to [Server](crate::server) and unless a connection error occurs, it does not
//! drop the connection. Timeouts are not supported yet.
//!
//! ```no_run
//! use {
//!   srpc::{client::Client, transport::Transport},
//!   std::sync::Arc,
//! };
//!
//! #[srpc::client]
//! trait StrService {
//!    async fn contains(data: String, elem: String) -> bool;
//!
//!    #[notification]
//!    async fn set_data(is_cool: bool);
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!    env_logger::init();
//!    let transporter = Arc::new(Transport::new());
//!    let client = Client::new(([127, 0, 0, 1], 8080).into(), transporter.clone());
//!
//!    for i in 0..100 {
//!        let _ = StrService::set_data(&client, i % 2 == 0).await;
//!        println!(
//!            "{}",
//!            StrService::contains(&client, String::from("cool lib"), String::from("lib"))
//!                .await
//!                .unwrap()
//!        );
//!    }
//! }
//! ```
//!

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
    sender: Arc<Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
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
            Some(sender) => sender.send(req)?,
            None => return Err(String::from("io error").into()),
        }
        Ok(rx.await?)
    }

    /// Makes an rpc notification call and DOES NOT wait for the response
    pub async fn notify(&self, request: json_rpc::Request) -> crate::Result<()> {
        self.handle_connection().await?;

        let res = self.create_data(&request)?;

        match self.sender.lock().await.as_mut() {
            Some(sender) => sender.send(res)?,
            None => return Err(String::from("io error").into()),
        }
        Ok(())
    }
}
