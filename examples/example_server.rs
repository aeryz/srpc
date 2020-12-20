use srpc::server::Server;

struct StrService;

#[srpc::service_impl]
impl StrService {
    async fn contains(data: String, elem: String) -> bool {
        data.contains(&elem)
    }

    async fn split_whitespace(data: String) -> Vec<String> {
        let mut v = Vec::new();
        for s in data.split_whitespace() {
            v.push(s.to_owned());
        }
        v
    }

    async fn foo(data: i32) -> i32 {
        5 + data
    }

    async fn bar(data: i32) -> i32 {
        6 + data
    }

    async fn no_args() -> String {
        String::new()
    }
}

struct NumService;

#[tokio::main]
async fn main() {
    let mut server = Server::new();
    // or
    server.add_service(Box::new(StrService::new()));
    server.add_service(Box::new(NumService::new()));
    server.remove_service(Box::new(NumService::new()));
    server.serve("127.0.0.1:8080").await;
}
