use srpc::client::Client;

//#[route = "str-service"]
#[srpc::client]
trait StrService {
    fn contains(data: String, elem: String) -> bool;

    fn split_whitespace(data: String) -> Vec<String>;

    fn foo();

    fn bar(n: i32);
}

/*
#[srpc::client]
#[route = "num-service"]
trait NumService {
    fn max(a: i32, b: i32) -> i32;

    fn factorial(n: u32) -> u32;
}
*/

fn main() {
    let mut client = Client::new("127.0.0.1:8080");
    match StrService::split_whitespace(&mut client, String::from("hello from haklsim"))     {
        Ok(res) => println!("{:?}", res),
        Err(e) => println!("Error {}", e),
    }
}
