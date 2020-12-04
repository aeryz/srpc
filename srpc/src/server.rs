use super::protocol::*;
use super::Result;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpListener;
use std::net::ToSocketAddrs;

pub trait Service {
    fn call<'a>(&self, fn_name: &'a str, args: serde_json::Value) -> Result<serde_json::Value>;

    fn get_route(&self) -> &'static str;
}

pub struct Server {
    services: HashMap<&'static str, Box<dyn Service>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn add_service(&mut self, service: Box<dyn Service>) -> Result<()> {
        if self.services.contains_key(service.get_route()) {
            return Err(String::new().into());
        }

        let route = service.get_route();
        self.services.insert(route, service);
        Ok(())
    }

    pub fn remove_service(&mut self, service: Box<dyn Service>) {
        self.services.remove(service.get_route());
    }

    // TODO: Return error if no service exists
    pub fn serve<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        use std::io::Read;
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            println!("Got a connection :)");
            let mut stream = stream?;
            let mut request = vec![0; 1024];
            let n_read = stream.read(&mut request)?;
            let req: SrpcRequest<serde_json::Value> = serde_json::from_slice(&request[0..n_read])?;
            let func = self.services.get(req.route).unwrap();
            let data = func.call(req.method_name, req.data)?;
            let response = SrpcResponse::new(StatusCode::SUCCESS, data);
            stream.write(serde_json::to_string(&response).unwrap().as_bytes())?;
        }

        Ok(())
    }
}
