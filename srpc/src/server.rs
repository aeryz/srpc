/// The async RPC server, which serves an RPC service.
///
/// Logic is simple. For each connection, server spawns a connection handler.
/// And the connection handler spawns a request handler for each request. Request
/// handlers run the corresponding RPC method and send a Response.
///
/// # Example
/// ```no_run
///
/// use srpc::server::Server;
///
/// struct MyService;
///
/// #[srpc::service]
/// impl MyService {
///     async fn contains(data: String, elem: String) -> bool {
///         data.contains(&elem)
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let server = Server::new(MyService::caller);
///     server.serve("127.0.0.1:8080").await;
/// }
/// ```
use {
    super::transport::Transport,
    crate::{json_rpc, transport::Reader},
    futures::stream::StreamExt,
    std::{future::Future, pin::Pin, sync::Arc},
    tokio::{
        io,
        net::{TcpListener, TcpStream, ToSocketAddrs},
        sync::mpsc,
    },
};

// An async function which returns an srpc::Result
type ServiceCall = fn(
    String,
    serde_json::Value,
) -> Pin<Box<dyn Future<Output = crate::Result<serde_json::Value>> + Send>>;

pub struct Server {
    service_call: ServiceCall,
    transport: Arc<Transport>,
}

impl Server {
    pub fn new(service_call: ServiceCall) -> Self {
        Self {
            service_call,
            transport: Arc::new(Transport::new()),
        }
    }

    pub fn set_service(&mut self, service_call: ServiceCall) {
        self.service_call = service_call;
    }

    /// Calls the corresponding rpc method and sends the result via sender.
    ///
    /// If an RPC error occurs, the error is sent in the 'error' field of the response and the 'result' field is
    /// set to 'None'.
    /// Otherwise the 'error' field is set to 'None' and the 'result' field contains the return
    /// value of the RPC method.
    async fn handle_request(
        self: Arc<Self>,
        request: json_rpc::Request,
        sender: mpsc::Sender<Vec<u8>>,
    ) {
        let value: Vec<u8> = match (self.service_call)(request.method, request.params).await {
            Ok(result) => json_rpc::Response::new_result(result, request.id.unwrap()),
            Err(_) => json_rpc::Response::new_error(
                json_rpc::ErrorKind::MethodNotFound,
                None,
                request.id.unwrap(),
            ),
        }
        .into();

        // TODO: Fix the unnecessary copy
        let mut response = Vec::from(value.len().to_le_bytes());
        response.extend(value);

        let _ = sender.send(response).await;
    }

    /// Spawns an IO reader and an IO writer for the connection and spawns new tasks as new
    /// requests come.
    async fn handle_connection(self: Arc<Self>, stream: TcpStream) {
        let (read_half, write_half) = io::split(stream);
        let mut reader: Reader<json_rpc::Request, _> = Reader::new(read_half);
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
