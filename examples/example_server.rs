use srpc::server::Server;

#[srpc::service(route = "str-service")]
struct StrService;

#[srpc::service_impl]
impl StrService {
    fn contains(data: String, elem: String) -> bool {
        data.contains(&elem)
    }

    fn split_whitespace(data: String) -> Vec<String> {
        let mut v = Vec::new();
        for s in data.split_whitespace() {
            v.push(s.to_owned());
        }
        v
    }

    fn foo() {
        println!("Hgeloo");
    }

    fn bar() -> () {
        println!("asd");
    }

    fn no_args() -> String {
        String::new()
    }
}

#[srpc::service(route = "num-service")]
struct NumService;

#[srpc::service_impl]
impl NumService {
    fn max(a: i32, b: i32) -> i32 {
        if a > b {
            a
        } else {
            b
        }
    }

    fn factorial(n: u32) -> u32 {
        match n {
            0 => 1,
            1 => 1,
            n => n * NumService::factorial(n - 1),
        }
    }
}

#[tokio::main]
async fn main() {
    let mut server = Server::new();
    // or
    server.add_service(Box::new(StrService::new()));
    server.add_service(Box::new(NumService::new()));
    server.remove_service(Box::new(NumService::new()));
    server.serve("127.0.0.1:8080").await;
}
