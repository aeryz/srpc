use super::protocol::*;
use super::Result;
use crate::json_rpc::*;
use crate::transport::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio::{net::TcpListener, sync::mpsc::channel};
use tokio::{net::ToSocketAddrs, sync::mpsc::Sender};

pub trait Service {
    fn call<'a>(&self, fn_name: &'a str, args: serde_json::Value) -> Result<serde_json::Value>;

    fn get_route(&self) -> &'static str;
}

pub struct Server {
    services: HashMap<&'static str, Box<dyn Service>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn add_service(&mut self, service: Box<dyn Service>) {
        if self.services.contains_key(service.get_route()) {
            return;
        }

        let route = service.get_route();
        self.services.insert(route, service);
    }

    pub fn remove_service(&mut self, service: Box<dyn Service>) {
        self.services.remove(service.get_route());
    }

    async fn handle_request<'a>(mut stream: TcpStream, tx: Sender<TransportData<TcpStream>>) {
        let mut total_data = Vec::new();

        loop {
            let mut data = [0 as u8; 1024];
            match stream.read(&mut data).await.unwrap() {
                0 => break,
                n => total_data.extend_from_slice(&data[0..n]),
            }
            if total_data.len() > 1
                && total_data[total_data.len() - 1] == b'\n'
                && total_data[total_data.len() - 2] == b'\r'
            {
                total_data.resize(total_data.len() - 2, 0);
                break;
            }
        }
        match Request::try_from(total_data.as_slice()) {
            Ok(request) => println!("{:?}", request),
            Err(e) => {
                let d = serde_json::to_vec(&e).unwrap();
                let data = TransportData { stream, data: d };
                let _ = tx.send(data).await;
            }
        }
    }

    // TODO: Return error if no service exists
    pub async fn serve<A: ToSocketAddrs>(&self, addr: A) {
        let listener = TcpListener::bind(addr).await.unwrap();

        let (tx, rx) = channel(32);
        let mut transport = Transport { receiver: rx };

        tokio::spawn(async move {
            transport.listen().await;
        });

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let tx2 = tx.clone();
            tokio::spawn(Self::handle_request(stream, tx2));
            /*
            let req: SrpcRequest<serde_json::Value> = serde_json::from_slice(&total_data).unwrap();
            let func = self.services.get(req.route).unwrap();
            let data = func.call(req.method_name, req.data).unwrap();
            let response = SrpcResponse::new(StatusCode::SUCCESS, data);
            stream
                .write(serde_json::to_string(&response).unwrap().as_bytes())
                .unwrap();
            */
        }
    }
}
