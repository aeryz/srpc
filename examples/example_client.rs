use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use srpc::{client::Client, json_rpc::Request};
use std::convert::TryFrom;

//#[srpc::client(route = "str-service")]
trait StrService {
    fn contains(data: String, elem: String) -> bool;

    fn split_whitespace(data: String) -> Vec<String>;

    fn foo();

    fn bar(n: i32);
}

//#[srpc::client(route = "num-service")]
trait NumService {
    fn max(a: i32, b: i32) -> i32;

    fn factorial(n: u32) -> u32;
}

use std::sync::Arc;
#[tokio::main]
async fn main() {
    let msg = "
        {
            \"jsonrpc\": \"2.0\",
            \"route\": \"test\",
            \"method\": \"foo\",
            \"params\": { \"data\": 1 },
            \"id\": 1
        }\r\n";

    let client = Client::new(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        8080,
    ));

    let r1 = Request::try_from(msg.as_bytes()).unwrap();
    let mut r2 = Request::try_from(msg.as_bytes()).unwrap();
    r2.method = String::from("bar");

    let f1 = client.clone().call(r1);
    let f2 = client.clone().call(r2);

    let (first, second) = tokio::join!(f1, f2);
    println!("{:?} {:?}", first, second);
    /*
    let res = StrService::split_whitespace(&mut client, String::from("hello from haksim"));
    println!("{:?}", res);
    */
}
