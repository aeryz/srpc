use serde::{Deserialize, Serialize};

struct MyServer;

#[srpc_macros::server_impl]
impl MyServer {
    fn foo(a: i32, b: i32) -> i32 {
        a + b
    }

    fn bar() -> String {
        String::new()
    }
}

/*
fn foo_expanded<'a>(args: String) -> impl serde::Serialize {
    #[derive(Deserialize)]
    struct Anon { a: i32, b: i32 };
    let args: Anon = serde_json::from_str(&args).unwrap();
    let (a, b) = (args.a, args.b);
    a + b
}
*/

fn main() {
    let server = MyServer;
    #[derive(Deserialize, Serialize, Debug)]
    struct Anon { a: i32, b: i32 };
    let s = serde_json::to_string(&Anon { a: 14, b: 15}).unwrap();

    let res: i32 = serde_json::from_str(&MyServer::foo(s)).unwrap();

    println!("{:?}", res);
}