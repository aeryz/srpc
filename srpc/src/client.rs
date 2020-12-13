use crate::utils;
use crate::{json_rpc::*, transport::*};
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

type RequestTable = Arc<Mutex<HashMap<Id, oneshot::Sender<Response>>>>;

pub struct Client {
    connection: Option<Arc<Mutex<TcpStream>>>,
    service_addr: SocketAddr,
    requests: RequestTable,
    sender: mpsc::Sender<TransportData<TcpStream>>,
}

impl Client {
    pub async fn new(service_addr: SocketAddr) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let requests = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(async move {
            let mut transport: Transport<TcpStream> = Transport::new(rx);
            transport.listen().await;
        });

        Self {
            connection: None,
            service_addr,
            requests: requests.clone(),
            sender: tx,
        }
    }

    async fn reader(requests: RequestTable, connection: Arc<Mutex<TcpStream>>) {
        while let Ok(res) = utils::read_frame(connection.clone()).await {
            let res = Response::try_from(res.as_slice()).unwrap();
            let mut requests = requests.lock().await;
            if let Some(sender) = requests.remove(&res.id) {
                let _ = sender.send(res);
            }
        }
    }

    async fn connect(&mut self) {
        self.connection = match self.connection.take() {
            Some(conn) => Some(conn),
            None => loop {
                match TcpStream::connect(self.service_addr).await {
                    Ok(conn) => {
                        let conn = Arc::new(Mutex::new(conn));
                        tokio::spawn(Client::reader(self.requests.clone(), conn.clone()));
                        break Some(conn);
                    }
                    Err(e) => {
                        eprintln!("Error occured: {}", e);
                        std::thread::sleep(Duration::new(1, 0));
                    }
                }
            },
        };
    }

    pub async fn call(&mut self, mut request: Request) -> Result<Response, ()> {
        self.connect().await;

        let sender = self.sender.clone();
        request.id = Some(Id::Num(rand::random::<u32>()));
        let (tx, mut rx) = oneshot::channel();

        self.requests
            .lock()
            .await
            .insert(request.id.clone().unwrap(), tx);

        let res = sender
            .send(TransportData::new(
                self.connection.as_ref().unwrap().clone(),
                serde_json::to_vec(&request).unwrap(),
            ))
            .await;

        if res.is_err() {
            return Err(());
        }

        rx.try_recv().map_err(|_| ())
    }
}
