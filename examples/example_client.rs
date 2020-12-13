use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use srpc::{client::Client, json_rpc::Request};

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

#[tokio::main]
async fn main() {
    let msg = "
        {
            \"jsonrpc\": \"2.0\",
            \"route\": \"test\",
            \"method\": \"foo\",
            \"params\": { \"data\": 1 },
            \"id\": 1,
        }\r\n";

    /*
    let res = StrService::split_whitespace(&mut client, String::from("hello from haksim"));
    println!("{:?}", res);
    */
}
