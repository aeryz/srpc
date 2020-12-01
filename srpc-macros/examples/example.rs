//use srpc::Server;

//#[route = "str-service"]
struct StrService;

#[srpc_macros::service]
impl StrService {
    fn contains(data: String, elem: String) -> bool {
        data.contains(&elem)
    }

    fn split_whitespace(data: String) -> Vec<String> {
        //data.split_whitespace().collect::<Vec<String>>()
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

/*
//#[route = "num-service"]
struct NumService;

#[srpc::service]
impl NumService {
    fn max(a: i32, b: i32) -> i32 {
        if a > b { a } else { b }
    }

    fn factorial(n: u32) -> u32 {
        match n {
            1 => 1,
            2 => 1,
            n => n * NumService::factorial(n - 1)
        }
    }
}
*/

fn main() {
    /*
    let server = Server::new(8080);
    server.serve(StrService::new());
    // or
    server.add_service(StrService::new());
    server.add_service(NumService::new());
    server.serve();
    */
}
