use srpc::Server;
use srpc::Service;

//#[route = "str-service"]
struct StrService {
    route: &'static str
}

impl StrService {
    fn new() -> Self { Self { route: "str-service" } }
}

#[srpc_macros::service]
impl StrService {
    fn contains(data: String, elem: String) -> bool {
        data.contains(&elem)
    }

    fn split_whitespace(data: String) -> Vec<String> {
        Vec::new()
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

//#[route = "num-service"]
struct NumService {
    route: &'static str,
}

impl NumService {
    fn new() -> Self { Self { route: "num-service" } }
}

#[srpc_macros::service]
impl NumService {
    fn max(a: i32, b: i32) -> i32 {
        if a > b { a } else { b }
    }

    fn factorial(n: u32) -> u32 {
        match n {
            0 => 1,
            1 => 1,
            n => n * NumService::factorial(n - 1)
        }
    }
}

fn main() {
    let mut server = Server::new(8080);
    // or
    server.add_service(Box::new(StrService::new()));
    server.add_service(Box::new(NumService::new()));
    server.serve();
}
