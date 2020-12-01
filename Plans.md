# Version 1.0 Plans

# API

## Server side

```rust

use srpc::Server;

#[route = "str-service"]
struct StrService;

#[srpc::service]
impl StrService {
    fn contains(data: String, elem: String) -> bool {
        data.contains(elem)
    }

    fn split_whitespace(data: String) -> Vec<String> {
        data.split_whitespace().collect::<Vec<String>>()
    }
}

#[route = "num-service"]
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
            n => n * factorial(n - 1)
        }
    }
}

fn main() {
    let server = Server::new(8080);
    server.add_service(StrService::new());
    server.add_service(NumService::new());
    server.serve(rpc::Protocol::HTTP);
}
```

### Defining a Service
Users need to define a struct for a service.
- Attributes:
    - route: Users can be able to serve multiple services in multiple routes. Default route is "/" and duplicate routes causes compilation error.
```rust
#[route = "str-service"]
struct StrService;
```

### Implementing a Service
srpc::service macro converts functions to async functions and makes all the networking and serialization operations under the hood.
A service's functions should not be called directly. It should only be used as a rpc service.
**FEATURE_REQUEST:** Provide an option to make service work both as a rpc service and local function.
```rust
#[srpc::service]
impl StrService {
    fn contains(data: String, elem: String) -> bool {
        data.contains(elem)
    }

    fn split_whitespace(data: String) -> Vec<String> {
        data.split_whitespace().collect::<Vec<String>>()
    }
}
```

### Starting the Server
```rust
fn new(port: u32) -> Result<rpc::Server>;
```
Returns a new server instance.

```rust
fn serve(protocol: rpc::Protocol) ->  Result<()>;
```
Serves the service based on the protocol (http, tcp, etc.). For now, only raw tcp and http protocols are supported.

```rust
fn add_service(service: impl rpc::Service) -> Result<rpc::Service>;
```
Adds a service to serve. Returns error if duplicate route or service is detected. This can also be called on the fly.

```rust
fn remove_service(service: impl rpc::Service);
```
Removes a service from server. This can also be called on the fly.

## Client Side
```rust
use srpc::Client;

#[srpc::client]
#[route = "str-service"]
trait StrService {
    fn contains(data: String, elem: String) -> bool;

    fn split_whitespace(data: String) -> Vec<String>;
}

#[srpc::client]
#[route = "num-service"]
trait NumService {
    fn max(a: i32, b: i32) -> i32;

    fn factorial(n: u32) -> u32;
}

fn main() {
    let client = Client::new("127.0.0.1:8080");
    StrService::contains(&mut client, String::new(), String::new()).await?;
    let connection = socket_connect().await?;
}
```

### Defining a Client

A service should be defined as a trait.
- Attributes
    - route: Specifies the route. Default is "/". 

```rust
#[srpc::client]
#[route = "str-service"]
trait StrService {
    fn contains(data: String, elem: String) -> bool;

    fn split_whitespace(data: String) -> Vec<String>;
}
```

### Functions
```rust
fn new(server_addr: &str) -> Result<srpc::Client>;
```
Returns a rpc client. Returns success if ip or port is invalid.

### Calling a RPC Function
Client parameter is added to the all rpc methods. Therefore, no service registration is needed.
```rust
ServiceName::function_name(&mut client, args).await?;
```
