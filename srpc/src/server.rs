use super::Result;
use crate::json_rpc::*;
use crate::transport::*;
use crate::utils;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use tokio::{io::AsyncWrite, net::TcpStream};
use tokio::{net::TcpListener, sync::mpsc::channel};
use tokio::{net::ToSocketAddrs, sync::mpsc::Sender};

type DynService = Box<dyn Service + Send + Sync>;
type ServiceMap = Arc<Mutex<HashMap<&'static str, DynService>>>;
type TMutex<T> = tokio::sync::Mutex<T>;

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

    async fn handle_request<T: AsyncWrite + Unpin>(
        stream: Arc<TMutex<T>>,
        request: Request,
        services: ServiceMap,
        sender: Sender<TransportData<T>>,
    ) {
        println!("Handling request");
        if !services
            .lock()
            .unwrap()
            .contains_key(request.route.as_str())
        {
            if request.id.is_none() {
                return;
            }
            utils::send_error_response(
                sender,
                stream,
                RpcErrorCode::METHOD_NOT_FOUND,
                "Route not found".to_owned(),
                None,
                request.id.clone().unwrap(),
            )
            .await;
        } else {
            let result = services
                .lock()
                .unwrap()
                .get(request.route.as_str())
                .unwrap()
                .call(request.method.as_str(), request.params);
            if request.id.is_none() {
                return;
            } else {
                match result {
                    Ok(result) => {
                        utils::send_result_response(
                            sender,
                            stream,
                            result,
                            request.id.clone().unwrap(),
                        )
                        .await;
                    }
                    Err(_) => {
                        utils::send_error_response(
                            sender,
                            stream,
                            RpcErrorCode::INTERNAL_ERRORS,
                            "Internal error".to_owned(),
                            None,
                            request.id.clone().unwrap(),
                        )
                        .await;
                    }
                }
            }
        }
    }

    pub async fn handle_connection(services: ServiceMap, stream: TcpStream) {
        let (tx, rx) = channel(32);

        tokio::spawn(async move {
            let mut transport = Transport::new(rx);
            transport.listen().await;
        });

        println!("Handling connection");

        let (read_half, write_half) = tokio::io::split(stream);

        let read_half = Arc::new(TMutex::new(read_half));
        let write_half = Arc::new(TMutex::new(write_half));

        loop {
            match Request::try_from(
                utils::read_frame(read_half.clone())
                    .await
                    .unwrap()
                    .as_slice(),
            ) {
                Ok(request) => {
                    tokio::spawn(Self::handle_request(
                        write_half.clone(),
                        request,
                        services.clone(),
                        tx.clone(),
                    ));
                }
                Err(_) => return,
            }
        }
    }

    pub async fn serve<A: ToSocketAddrs>(&self, addr: A) {
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let services = self.services.clone();
            tokio::spawn(Self::handle_connection(services, stream));
        }
    }
}
