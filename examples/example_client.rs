use srpc::Client;

#[srpc::client]
#[route = "str-service"]
trait StrService {
    fn contains(data: String, elem: String) -> bool;

    fn split_whitespace(data: String) -> Vec<String>;
}

#[srpc::client]
#[route = "num-service"]
trait NumService {
    fn max(a: i32, b: i32) -> i32;

    fn factorial(n: u32) -> u32;
}

fn main() {
    let client = Client::new("127.0.0.1:8080");
    client.i_got_dis(json);
    client.raw_call("num-service", "contains", json).await?;
    client.rpc.contains(String::new(), String::new()).await?;
    let connection = socket_connect().await?;
}
