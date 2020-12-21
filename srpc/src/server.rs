use {
    super::{transport::Transport, Result},
    crate::{json_rpc::*, transport::Reader},
    futures::stream::StreamExt,
    std::{future::Future, pin::Pin, sync::Arc},
    tokio::{
        io,
        net::{TcpListener, TcpStream, ToSocketAddrs},
        sync::mpsc,
    },
};

type ServiceCall = fn(
    String,
    serde_json::Value,
) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>;

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

    async fn handle_request(self: Arc<Self>, request: Request, sender: mpsc::Sender<Vec<u8>>) {
        let value: Vec<u8> = match (self.service_call)(request.method, request.params).await {
            Ok(result) => Response::new_result(result, request.id.unwrap()),
            Err(_) => Response::new_error(ErrorKind::MethodNotFound, None, request.id.unwrap()),
        }
        .into();

        let mut response = Vec::from(value.len().to_le_bytes());
        response.extend(value);

        let _ = sender.send(response).await;
    }

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
