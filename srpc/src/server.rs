//! The async RPC server.
//!
//! Logic is simple. For each connection, server spawns a connection handler.
//! And the connection handler spawns a request handler for each request. Request
//! handlers run the corresponding RPC method and send back a Response(if the request
//! is not a notification). Unless any error occurs related to connection, the
//! server keeps the connection open.
//!
//! # Example
//! ```no_run
//! use srpc::server::Context;
//! use srpc::server::Server;
//! use std::sync::Arc;
//!
//! struct StrService;
//!
//! #[srpc::service]
//! #[allow(unused)]
//! impl StrService {
//!    async fn contains(data: String, elem: String) -> bool {
//!        data.contains(&elem)
//!    }
//!
//!    async fn set_data(context: Arc<Context>, is_cool: bool) {
//!        println!("Socket {:?}", context.caller_addr);
//!        println!("Set a cool variable to: {}", is_cool);
//!    }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let server = Server::new(StrService, MyService::caller);
//!     let _ = server.serve("127.0.0.1:8080").await;
//! }
//! ```
//!
//! # Reserved Parameters
//!
//! Services might want to use some local data which is not sent to the client. Currently,
//! there are two reserved parameters which are passed to the service method if they present
//! in the function parameters.
//!
//! ## Local server data
//! Whenever shared server data is needed, this parameter can be used. SRPC passes the same data
//! to every method so the data is not copied. Users do not pay the cost comes with `Arc` if they
//! don't use the parameter.
//! ```no_run
//! async fn foo(self: Arc<Self>) {}
//! ```
//!
//! ## Context of the connection
//! Server might wanna know where the connection comes from. In that case `context: Arc<Context>`
//! is used. For now, [Context](struct.Context.html) is only contains the address of the connector
//! client.
//! ```no_run
//! async fn foo(context: Arc<Context>) {}
//! ```
//!
//!
use {
    super::transport::Transport,
    crate::{
        json_rpc,
        transport::{codec, Reader},
    },
    futures::stream::StreamExt,
    std::{future::Future, net::SocketAddr, pin::Pin, sync::Arc},
    tokio::{
        io,
        net::{TcpListener, TcpStream, ToSocketAddrs},
        sync::mpsc,
    },
};

// An async function which returns an srpc::Result
type ServiceCall<T> =
    fn(
        Arc<T>,
        Arc<Context>,
        String,
        serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, json_rpc::Error>> + Send>>;

#[derive(Copy, Clone)]
pub struct Context {
    pub caller_addr: SocketAddr,
}

pub struct Server<T> {
    service: Arc<T>,
    service_call: ServiceCall<T>,
    transport: Arc<Transport>,
}

impl<T> Server<T>
where
    T: 'static + Send + Sync,
{
    pub fn new(service: T, service_call: ServiceCall<T>) -> Self {
        Self {
            service: Arc::new(service),
            service_call,
            transport: Arc::new(Transport::new()),
        }
    }

    pub fn set_service(&mut self, service: T, service_call: ServiceCall<T>) {
        self.service = Arc::new(service);
        self.service_call = service_call;
    }

    /// Calls the corresponding rpc method and sends the result via sender. If the request is a
    /// notification, no data is sent back.
    async fn handle_single_request(
        self: &Arc<Self>,
        context: Arc<Context>,
        request: json_rpc::Request,
        sender: mpsc::UnboundedSender<Vec<u8>>,
    ) {
        if let Some(id) = request.id {
            let response: Vec<u8> = match (self.service_call)(
                self.service.clone(),
                context,
                request.method,
                request.params,
            )
            .await
            {
                Ok(result) => json_rpc::Response::from_result(result, id),
                Err(err) => json_rpc::Response::from_error(err, id),
            }
            .into();

            if response.len() > std::u32::MAX as usize {
                panic!("maximum response size is exceeded");
            }
            // TODO: Error handling
            let _ = sender.send(response);
        } else {
            // We don't need to see the result of a notification
            let _ = (self.service_call)(
                self.service.clone(),
                context,
                request.method,
                request.params,
            )
            .await;
        }
    }

    /// Calls the corresponding rpc method for each request and sends the results of each
    /// non-notification RPC call at once, as an array. It sends back an empty array, if all requests are
    /// notifications.
    async fn handle_batched_request(
        self: &Arc<Self>,
        context: Arc<Context>,
        requests: Vec<json_rpc::Request>,
        sender: mpsc::UnboundedSender<Vec<u8>>,
    ) {
        let mut response = vec![b'['];
        for request in requests {
            // Is it a notification?
            if let Some(id) = request.id {
                let value: Vec<u8> = match (self.service_call)(
                    self.service.clone(),
                    context.clone(),
                    request.method,
                    request.params,
                )
                .await
                {
                    Ok(result) => json_rpc::Response::from_result(result, id),
                    Err(err) => json_rpc::Response::from_error(err, id),
                }
                .into();
                response.extend(value);
                response.push(b',');
            } else {
                // We don't need the result of a notification
                let _ = (self.service_call)(
                    self.service.clone(),
                    context.clone(),
                    request.method,
                    request.params,
                )
                .await;
            }
        }
        if response.len() != 1 {
            if response.len() > u32::MAX as usize {
                panic!("maximum response size is exceeded");
            }
            response.pop();
        }
        response.push(b']');

        let _ = sender.send(response);
    }

    /// Calls the appropriate request handler
    ///
    /// If an RPC error occurs, the error is sent in the 'error' field of the response and the
    /// 'result' field is set to 'None'.
    /// Otherwise the 'error' field is set to 'None' and the 'result' field contains the return
    /// value of the RPC method.
    async fn handle_request(
        self: Arc<Self>,
        context: Arc<Context>,
        request: codec::Type<json_rpc::Request>,
        sender: mpsc::UnboundedSender<Vec<u8>>,
    ) {
        match request {
            codec::Type::Single(request) => {
                self.handle_single_request(context, request, sender).await
            }
            codec::Type::Batched(requests) => {
                self.handle_batched_request(context, requests, sender).await
            }
        };
    }

    /// Spawns an IO reader and an IO writer for the connection and spawns new tasks as new
    /// requests come.
    async fn handle_connection(self: Arc<Self>, stream: TcpStream, context: Context) {
        log::debug!("Handling the connection from {:?}", stream.local_addr());
        let (read_half, write_half) = io::split(stream);
        let mut reader: Reader<json_rpc::Request, _> = Reader::new(read_half);
        let sender = self.transport.spawn_writer(write_half);
        let context = Arc::new(context);

        loop {
            match reader.next().await {
                Some(Ok(request)) => {
                    let sender_clone = sender.clone();
                    let self_clone = self.clone();
                    let context_clone = context.clone();
                    tokio::spawn(async move {
                        Self::handle_request(self_clone, context_clone, request, sender_clone)
                            .await;
                    });
                }
                Some(Err(e)) => {
                    log::error!("Error occured during handling connection: {}", e);
                    break;
                }
                None => break,
            }
        }
    }

    /// Serves services from a TcpStream.
    /// When a new connection is accepted, it spawns a task to handle that connection.
    ///
    /// TODO: Server is limited to TcpStream right now. It should be able to serve
    ///       anything that implements Stream trait
    pub async fn serve<A: ToSocketAddrs>(self, addr: A) -> crate::Result<()> {
        let listener = TcpListener::bind(addr).await?;

        let arc_self = Arc::new(self);
        loop {
            let (stream, addr) = listener.accept().await?;
            let context = Context { caller_addr: addr };
            let self_clone = arc_self.clone();
            tokio::spawn(async move { self_clone.handle_connection(stream, context).await });
        }
    }
}
