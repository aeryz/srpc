use crate::{
    json_rpc::*,
    transport::{reader::Reader, writer::PersistantWriter},
};
use futures::stream::StreamExt;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::{collections::HashMap, time::Duration};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::oneshot;

type RequestTable = Mutex<HashMap<Id, oneshot::Sender<Response>>>;
type ReaderType = Reader<Response, ReadHalf<TcpStream>>;
type WriterType = PersistantWriter<WriteHalf<TcpStream>>;
type RequestQueue = Mutex<VecDeque<Vec<u8>>>;
type RequestCond = (Mutex<usize>, Condvar);

pub struct Client {
    service_addr: SocketAddr,
    req_table: RequestTable,
    req_queue: RequestQueue,
    req_cond: RequestCond,
}

impl Client {
    pub fn new(service_addr: SocketAddr) -> Arc<Self> {
        let client = Arc::new(Self {
            service_addr,
            req_table: Mutex::new(HashMap::new()),
            req_queue: Mutex::new(VecDeque::new()),
            req_cond: (Mutex::new(0), Condvar::new()),
        });

        let client_clone = client.clone();
        std::thread::spawn(move || client_clone.handle_connection());
        client
    }

    async fn reader<R: AsyncRead + Unpin>(&self, r: R) {
        let mut reader: Reader<Response, R> = Reader::new(r);
        while let Some(data) = reader.next().await {
            if let Ok(res) = data {
                //println!("Read: {}", String::from_utf8(res.clone()).unwrap());
                let mut req_table = self.req_table.lock().unwrap();
                if let Some(sender) = req_table.remove(&res.id) {
                    let _ = sender.send(res);
                }
            }
        }
    }

    async fn writer<W: AsyncWrite + Unpin>(&self, mut writer: W) {
        loop {
            let (lock, cvar) = &self.req_cond;
            {
                let mut not_empty = lock.lock().unwrap();
                while *not_empty == 0 {
                    not_empty = cvar.wait(not_empty).unwrap();
                }
            }
            let mut start_pos = 0;
            let data = self.req_queue.lock().unwrap().pop_back().unwrap();
            let data_len = data.len();
            while let Ok(n) = writer.write(&data[start_pos..data_len]).await {
                if n == 0 {
                    break;
                }
                start_pos += n;
            }
        }
    }

    fn handle_connection(self: Arc<Self>) {
        match TcpStream::connect(self.service_addr) {
            Ok(conn) => {
                let (r, w) = tokio::io::split(conn);
                tokio::join!(self.reader(r), self.writer(w));
            }
            Err(e) => {
                println!("Connection error: {}", e);
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }

    pub async fn call(self: Arc<Self>, mut request: Request) -> Result<Response, ()> {
        request.id = Some(Id::Num(rand::random::<u32>()));
        let (tx, rx) = oneshot::channel();
        let res = tokio::spawn(rx);

        self.req_table
            .lock()
            .unwrap()
            .insert(request.id.clone().unwrap(), tx);

        let mut data_to_send = serde_json::to_vec(&request).unwrap();
        data_to_send.push(b'\r');
        data_to_send.push(b'\n');

        let mut req_queue = self.req_queue.lock().unwrap();
        req_queue.push_back(data_to_send);

        let (lock, cvar) = &self.req_cond;
        *lock.lock().unwrap() = req_queue.len();
        cvar.notify_one();

        match res.await {
            Ok(res) => res.map_err(|_| ()),
            Err(_) => Err(()),
        }
    }
}
