use crate::utils;
use crate::{json_rpc::*, transport::*};
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

type RequestTable = Arc<Mutex<HashMap<Id, oneshot::Sender<Response>>>>;

pub struct Connection {
    read_end: Arc<Mutex<ReadHalf<TcpStream>>>,
    write_end: Arc<Mutex<WriteHalf<TcpStream>>>,
}

pub struct Client {
    connection: Arc<Mutex<Option<Connection>>>,
    service_addr: SocketAddr,
    requests: RequestTable,
    sender: mpsc::Sender<TransportData<WriteHalf<TcpStream>>>,
}

impl Client {
    pub async fn new(service_addr: SocketAddr) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let requests = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(async move {
            let mut transport: Transport<WriteHalf<TcpStream>> = Transport::new(rx);
            transport.listen().await;
        });

        Self {
            connection: Arc::new(Mutex::new(None)),
            service_addr,
            requests: requests.clone(),
            sender: tx,
        }
    }

    async fn reader<T: AsyncReadExt + Unpin>(requests: RequestTable, reader: Arc<Mutex<T>>) {
        println!("Reader is ready");
        while let Ok(res) = utils::read_frame(reader.clone()).await {
            println!("Read: {}", String::from_utf8(res.clone()).unwrap());
            let res = Response::try_from(res.as_slice()).unwrap();
            let mut requests = requests.lock().await;
            if let Some(sender) = requests.remove(&res.id) {
                let _ = sender.send(res);
            }
        }
    }

    async fn connect(&self) {
        let mut connection = self.connection.lock().await;
        if connection.is_some() {
            println!("Reusing connection");
            return;
        }

        loop {
            match TcpStream::connect(self.service_addr).await {
                Ok(conn) => {
                    let (r, w) = tokio::io::split(conn);
                    *connection = Some(Connection {
                        read_end: Arc::new(Mutex::new(r)),
                        write_end: Arc::new(Mutex::new(w)),
                    });
                    tokio::spawn(Client::reader(
                        self.requests.clone(),
                        connection.as_ref().unwrap().read_end.clone(),
                    ));
                    break;
                }
                Err(e) => {
                    eprintln!("Error occured: {}", e);
                    std::thread::sleep(Duration::new(1, 0));
                }
            }
        }
    }

    pub async fn call(&self, mut request: Request) -> Result<Response, ()> {
        self.connect().await;

        let sender = self.sender.clone();
        request.id = Some(Id::Num(rand::random::<u32>()));
        let (tx, rx) = oneshot::channel();

        let res = tokio::spawn(rx);

        self.requests
            .lock()
            .await
            .insert(request.id.clone().unwrap(), tx);

        let mut data_to_send = serde_json::to_vec(&request).unwrap();
        data_to_send.push(b'\r');
        data_to_send.push(b'\n');

        let send_res = sender
            .send(TransportData::new(
                self.connection
                    .lock()
                    .await
                    .as_ref()
                    .unwrap()
                    .write_end
                    .clone(),
                data_to_send,
            ))
            .await;

        if send_res.is_err() {
            return Err(());
        }

        match res.await {
            Ok(res) => res.map_err(|_| ()),
            Err(_) => Err(()),
        }
    }
}
