use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Client;

impl Client {
    pub fn new(_ip_addr: &str) -> Self {
        Self
    }

    pub fn call(&mut self) {
        let mut connection = TcpStream::connect("localhost:8080").unwrap();
        let msg = "
            {
                \"jsonrpc\": \"2.0\",
                \"route\": \"str-service\",
                \"method\": \"split_whitespace\",
                \"params\": {
                    \"data\": \"Hello from haksim\"
                },
                \"id\": 1
            }\r\n";

        connection.write(msg.as_bytes()).unwrap();
        let mut resp = vec![0; 1024];
        let n_read = connection.read(&mut resp).unwrap();
        resp.resize(n_read, 0);
        println!("{}", String::from_utf8(resp).unwrap());
    }
}
