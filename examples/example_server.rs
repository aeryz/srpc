use srpc::server::Context;
use srpc::server::Server;
use std::sync::Arc;

struct StrService;

#[srpc::service]
impl StrService {
    async fn contains(data: String, elem: String) -> bool {
        data.contains(&elem)
    }

    async fn set_data(context: Arc<Context>, is_cool: bool) {
        println!("Socket {:?}", context.caller_addr);
        println!("Set a cool variable to: {}", is_cool);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let server = Server::new(StrService, StrService::caller);
    let _ = server.serve("127.0.0.1:8080").await;
}
