use srpc::server::Server;

struct StrService;

#[srpc::service]
impl StrService {
    async fn contains(data: String, elem: String) -> bool {
        data.contains(&elem)
    }

    async fn set_data(is_cool: bool) {
        println!("Setted a cool variable to: {}", is_cool);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let server = Server::new(StrService::caller);
    let _ = server.serve("127.0.0.1:8080").await;
}
