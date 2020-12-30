# Simple JSON-RPC 
SRPC is a high level and asynchronous JSON-RPC client and server library which aims for ease of use. It only supports transportation over TCP sockets for now.

# Note
SRPC is in the early development phase.

# Usage

## Server side

```rust
use srpc::server::Server;
use std::sync::Arc;

struct StrService {
	// Some server data
};

#[srpc::service]
impl StrService {
	// 'self' parameter is used as local server data. It is not transferred.
    async fn contains(self: Arc<Self>, data: String, elem: String) -> bool {
        data.contains(&elem)
    }

    async fn set_data(self: Arc<Self>, is_cool: bool) {
        println!("Set a cool variable to: {}", is_cool);
    }
}

#[tokio::main]
async fn main() {
    let server = Server::new(StrService{ /* data */ }, StrService::caller);
    let _ = server.serve("127.0.0.1:8080").await;
}
```

## Client side
```rust
 use {
     srpc::{client::Client, transport::Transport},
     std::{
         net::{IpAddr, Ipv4Addr, SocketAddr},
         sync::Arc,
     },
 };
 
 #[srpc::client]
 trait StrService {
     // Define an RPC call, that expects and waits for a response.
     async fn contains(data: String, elem: String) -> bool;
 
     #[notification] // Define an RPC notification that does not wait for a response.
     async fn set_data(is_cool: bool);
 }
 
 #[tokio::main]
 async fn main() {
     let transporter = Arc::new(Transport::new());
     let client = Client::new(
         SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
         transporter.clone(),
     );
          
     let _ = StrService::set_data(&client, true).await;
     let res = StrService::contains(&client, String::from("cool lib"), String::from("lib"))
                 .await
                 .unwrap();
     println!("{}", res);
 }
```

# Current Version Supports (0.1.0)
- JSON-RPC 2.0
- Persistent connection
- Batched requests on the server side
- Asyncronous RPC methods
- Tokio as runtime

# Future Releases
- Batched requests on the client side (0.2.0)
- Blocking annotation for blocking RPC methods (0.2.0)
- Executor agnostic (0.3.0)
- Data framing over TCP (0.3.0)
- JSON-RPC 1.0 (after 0.3.0)

# Contribution
I don't have a contribution guideline yet, so feel free to go to issues, pick one and send a PR :) Easy ones are marked with "good first issue" label.
