use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use srpc::client::Client;
use srpc::transport::Transport;

#[srpc::client]
trait StrService {
    async fn foo(data: i32) -> i32;
}

use std::sync::Arc;

#[tokio::main]
async fn main() {
    let transporter = Arc::new(Transport::new());
    let client = Client::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        transporter.clone(),
    );

    for _ in 0..10000 {
        println!("{}", StrService::foo(&client, 5).await.unwrap());
    }
}

/*
#[tokio::main]
async fn main() {
    let msg = "
        {
            \"jsonrpc\": \"2.0\",
            \"method\": \"foo\",
            \"params\": { \"data\": 1 },
            \"id\": 1
        }\r\n";

    let transporter = Arc::new(Transport::new());
    let client = Client::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        transporter.clone(),
    );

    let r1 = Request::try_from(msg.as_bytes()).unwrap();
    let mut r2 = Request::try_from(msg.as_bytes()).unwrap();
    let mut r3 = Request::try_from(msg.as_bytes()).unwrap();
    let mut r4 = Request::try_from(msg.as_bytes()).unwrap();
    let mut r5 = Request::try_from(msg.as_bytes()).unwrap();

    let f1 = client.call(r1);
    let f2 = client.call(r2);
    let f3 = client.call(r3);
    let f4 = client.call(r4);
    let f5 = client.call(r5);

    let (r1, r2, r3, r4, r5) = tokio::join!(f1, f2, f3, f4, f5);
    println!("1: {:?}", r1);
    println!("2: {:?}", r2);
    println!("3: {:?}", r3);
    println!("4: {:?}", r4);
    println!("5: {:?}", r5);
    /*
    let res = StrService::split_whitespace(&mut client, String::from("hello from haksim"));
    println!("{:?}", res);
    */
}
*/
