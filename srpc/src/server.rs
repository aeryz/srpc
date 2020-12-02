use std::collections::HashMap;
use super::Result;
use std::net::TcpListener;
use std::io::Write;

pub trait Service {
    fn call(&self, fn_name: String, args: String) -> Result<String>;

    fn get_route(&self) -> &'static str;
}

pub struct Server {
    port: u32,
    services: HashMap<&'static str, Box<dyn Service>>,
}

#[derive(serde::Deserialize)]
struct Args {
    route: String,
    method_name: String,
    args: String
}

impl Server {
    pub fn new(port: u32) -> Self {
        Self {
            port,
            services: HashMap::new()
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
    pub fn serve(&self) -> Result<()> {
        use std::io::Read;
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        for stream in listener.incoming() {
            println!("Got a connection :)");
            let mut stream = stream?;
            let mut request = vec![0; 1024];
            let n_read = stream.read(&mut request)?;
            let args: Args = serde_json::from_slice(&request[0..n_read])?;
            let func = self.services.get(args.route.as_str()).unwrap();
            let data = func.call(args.method_name, args.args)?;
            stream.write(data.as_bytes());
        }

        Ok(())
    }
}