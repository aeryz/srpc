use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use srpc::client::Client;
use srpc::transport::Transport;

#[srpc::client]
trait StrService {
    async fn contains(data: String, elem: String) -> bool;
}

use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::init();
    let transporter = Arc::new(Transport::new());
    let client = Client::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        transporter.clone(),
    );

    for _ in 0..100 {
        println!(
            "{}",
            StrService::contains(&client, String::from("cool lib"), String::from("lib"))
                .await
                .unwrap()
        );
    }
}
