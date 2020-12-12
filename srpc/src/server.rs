use super::Result;
use crate::json_rpc::*;
use crate::transport::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio::{net::TcpListener, sync::mpsc::channel};
use tokio::{net::ToSocketAddrs, sync::mpsc::Sender};

type DynService = Box<dyn Service + Send + Sync>;
type ServiceMap = Arc<Mutex<HashMap<&'static str, DynService>>>;

pub trait Service {
    fn call(&self, fn_name: &str, args: serde_json::Value) -> Result<serde_json::Value>;

    fn get_route(&self) -> &'static str;
}

pub struct Server {
    services: ServiceMap,
}

impl Server {
    pub fn new() -> Self {
        Self {
            services: ServiceMap::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_service(&mut self, service: DynService) {
        let mut services = self.services.lock().unwrap();
        if services.contains_key(service.get_route()) {
            return;
        }

        let route = service.get_route();
        services.insert(route, service);
    }

    pub fn remove_service(&mut self, service: DynService) {
        self.services.lock().unwrap().remove(service.get_route());
    }

    async fn handle_request(
        services: ServiceMap,
        mut stream: TcpStream,
        tx: Sender<TransportData<TcpStream>>,
    ) {
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
            Ok(request) => {
                if !services
                    .lock()
                    .unwrap()
                    .contains_key(request.route.as_str())
                {
                    if request.id.is_none() {
                        return;
                    }
                    let response = Response {
                        jsonrpc: "2.0".to_owned(),
                        result: None,
                        error: Some(RpcError::new(
                            RpcErrorCode::METHOD_NOT_FOUND,
                            "Route not found.".to_owned(),
                            None,
                        )),
                        id: request.id.clone().unwrap(),
                    };
                    let _ = tx.send(TransportData {
                        stream,
                        data: serde_json::to_vec(&response).unwrap(),
                    });
                } else {
                    let response = {
                        let value = services
                            .lock()
                            .unwrap()
                            .get(request.route.as_str())
                            .unwrap()
                            .call(request.method.as_str(), request.params);
                        if request.id.is_none() {
                            return;
                        }
                        let response = match value {
                            Ok(value) => Response {
                                jsonrpc: "2.0".to_owned(),
                                result: Some(value),
                                error: None,
                                id: request.id.unwrap(),
                            },
                            Err(_) => Response {
                                jsonrpc: "2.0".to_owned(),
                                result: None,
                                error: Some(RpcError::new(
                                    RpcErrorCode::INTERNAL_ERRORS,
                                    "Internal error".to_owned(),
                                    None,
                                )),
                                id: request.id.unwrap(),
                            },
                        };
                        response
                    };
                    //println!("{:?}", response);
                    let _ = tx
                        .send(TransportData {
                            stream,
                            data: serde_json::to_vec(&response).unwrap(),
                        })
                        .await;
                }
            }
            Err(e) => {
                let d = serde_json::to_vec(&e).unwrap();
                let data = TransportData { stream, data: d };
                let _ = tx.send(data).await;
            }
        }
    }

    // TODO: Return error if no service exists
    pub async fn serve<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;

        let (tx, rx) = channel(32);
        let mut transport = Transport { receiver: rx };

        tokio::spawn(async move {
            transport.listen().await;
        });

        loop {
            let (stream, _) = listener.accept().await?;
            let tx2 = tx.clone();
            let services = self.services.clone();
            tokio::spawn(Self::handle_request(services, stream, tx2));
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
