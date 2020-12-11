use super::protocol::*;
use super::Result;
use serde::Serialize;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Client;

impl Client {
    pub fn new(_ip_addr: &str) -> Self {
        Self
    }

    pub fn call<'a, Req: Serialize>(&mut self, request: SrpcRequest<Req>) -> Result<Vec<u8>> {
        let mut connection = TcpStream::connect("localhost:8080")?;
        let msg = serde_json::to_vec(&request)?;
        connection.write(&msg)?;
        let mut resp = vec![0; 1024];
        let n_read = connection.read(&mut resp)?;
        println!("Read: {} bytes", n_read);
        resp.resize(n_read, 0);

        Ok(resp)
    }

    pub fn call2(&mut self, test: bool) {
        let mut connection = TcpStream::connect("localhost:8080").unwrap();
        let msg = "
            {
                \"jsonrpc\": \"2.0\",
                \"method\": \"\",
                \"params\": \"\"
            }\r\n";
        let msg2 = "
            {
                \"jsonrpc\": \"2.0\",
                \"method\": \"\",
                \"sparams\": \"\"
            }\r\n";

        if test {
            connection.write(msg.as_bytes()).unwrap();
        } else {
            connection.write(msg2.as_bytes()).unwrap();
            let mut resp = vec![0; 1024];
            let n_read = connection.read(&mut resp).unwrap();
            resp.resize(n_read, 0);
            println!("{}", String::from_utf8(resp).unwrap());
        }
    }
}
