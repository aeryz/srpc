use srpc::client::Client;

#[srpc::client(route = "str-service")]
trait StrService {
    fn contains(data: String, elem: String) -> bool;

    fn split_whitespace(data: String) -> Vec<String>;

    fn foo();

    fn bar(n: i32);
}

#[srpc::client(route = "num-service")]
trait NumService {
    fn max(a: i32, b: i32) -> i32;

    fn factorial(n: u32) -> u32;
}

fn main() {
    let mut client = Client::new("127.0.0.1:8080");
    client.call2(true);
    client.call2(false);
    /*
    let res = StrService::split_whitespace(&mut client, String::from("hello from haksim"));
    println!("{:?}", res);
    */
}
