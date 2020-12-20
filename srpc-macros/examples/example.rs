//use srpc::Server;

#[srpc_macros::client]
trait StrService {
    fn foo(data: i32) -> i32;
}

fn main() {
    /*
    let mut server = Server::new(StrService);
    server.serve("127.0.0.1:8080").await;
    */
}
