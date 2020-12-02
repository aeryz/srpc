use super::Result;
use std::net::TcpStream;
use std::io::{Write, Read};

pub struct Client;

#[derive(serde::Serialize)]
struct Args {
    route: &'static str,
    method_name: &'static str,
    args: String
}

impl Client {
    pub fn new(_ip_addr: &str) -> Self {
        Self
    }

    pub fn call(&mut self, route: &'static str, method_name: &'static str, args: String) -> Result<Vec<u8>> {
        println!("{}/{} is called with args: \n{}", route, method_name, args);
        let mut connection = TcpStream::connect("localhost:8080")?;
        println!("Connected to server");
        let msg = serde_json::to_string(&Args { route, method_name, args })?.into_bytes();
        connection.write(&msg)?;
        let mut resp = vec![0; 1024];
        let n_read = connection.read(&mut resp)?;
        println!("Read: {} bytes", n_read);
        resp.resize(n_read, 0);
        Ok(resp)
    }
}