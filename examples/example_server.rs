use srpc::server::Server;

struct StrService;

#[srpc::service]
impl StrService {
    async fn contains(data: String, elem: String) -> bool {
        data.contains(&elem)
    }

    async fn set_data(is_cool: bool) {
        println!("Setted a cool variable to: {}", is_cool);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let server = Server::new(StrService::caller);
    let _ = server.serve("127.0.0.1:8080").await;
}

/*
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
*/
