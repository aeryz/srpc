use super::transport::Transport;
use super::Result;
use crate::json_rpc::*;
use crate::transport::Reader;
use crate::utils;
use async_trait::async_trait;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{self, AsyncWrite};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;

/// This trait is auto-implemented by the 'service_impl' macro.
#[async_trait]
pub trait Service {
    /// Calls the appropriate rpc function and returns its value as `serde_json::Value`
    ///
    /// # Errors
    /// TODO
    async fn call(fn_name: String, args: serde_json::Value) -> Result<serde_json::Value>;
}

pub struct Server<S> {
    service: S,
    transport: Arc<Transport>,
}

impl<S> Server<S>
where
    S: Service + Send + Sync + 'static,
{
    pub fn new(service: S) -> Self {
        Self {
            service,
            transport: Arc::new(Transport::new()),
        }
    }

    pub fn set_service(&mut self, service: S) {
        self.service = service;
    }

    async fn handle_request(self: Arc<Self>, request: Request, sender: mpsc::Sender<Vec<u8>>) {}

    async fn handle_connection(self: Arc<Self>, stream: TcpStream) {
        let (read_half, write_half) = io::split(stream);
        let mut reader: Reader<Request, _> = Reader::new(read_half);
        let sender = self.transport.spawn_writer(write_half);

        loop {
            match reader.next().await {
                Some(Ok(request)) => {
                    println!("Spawning a task to handle the request");
                    let sender_clone = sender.clone();
                    let self_clone = self.clone();
                    tokio::spawn(async move {
                        Self::handle_request(self_clone, request, sender_clone).await;
                    });
                }
                Some(Err(e)) => {
                    println!("Error occured during handling connection: {}", e);
                }
                None => break,
            }
        }
    }

    /// Serves services from a TcpStream for now, it should accept all kind of type
    /// which implements the Stream trait and the other necessary traits.
    /// When a new connection is accepted, it spawns a task to handle that connection.
    pub async fn serve<A: ToSocketAddrs>(self, addr: A) {
        let listener = TcpListener::bind(addr).await.unwrap();

        let arc_self = Arc::new(self);
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let self_clone = arc_self.clone();
            tokio::spawn(async move { self_clone.handle_connection(stream).await });
        }
    }
}
