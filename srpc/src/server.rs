use std::collections::HashMap;
use super::Result;

pub trait Service {
    fn call(&self, fn_name: String, args: String) -> Result<String>;

    fn get_route(&self) -> &'static str;
}

pub struct Server {
    port: u32,
    services: HashMap<&'static str, Box<dyn Service>>,
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
        loop {
            let mut buffer = String::new();
            println!("RPC route:");
            std::io::stdin().read_line(&mut buffer).unwrap();
            buffer.pop();
            let args = buffer.split("/").collect::<Vec<&str>>();
            let func = self.services.get(args[0]);
            if let Some(func) = func {
                match func.call(args[1].to_owned(), args[2].to_owned()) {
                    Ok(ret) => println!("{}", ret),
                    Err(e) => println!("{}", e.to_string()),
                }
            } else {
                println!("RPC not found.");
            }
        }
    }
}