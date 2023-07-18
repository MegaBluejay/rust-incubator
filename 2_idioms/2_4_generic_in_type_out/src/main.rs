use std::net::{IpAddr, SocketAddr};

use default_struct_builder::DefaultBuilder;
use smart_default::SmartDefault;

fn main() {
    println!("Refactor me!");

    let _err = Error::default()
        .code("NO_USER".to_string())
        .status(404)
        .message("User not found".to_string());
}

#[derive(Debug, SmartDefault, DefaultBuilder)]
pub struct Error {
    #[builder(into)]
    #[default = "UNKNOWN"]
    code: String,
    #[default = 500]
    status: u16,
    #[builder(into)]
    #[default = "Unknown error has happened"]
    message: String,
}

#[derive(Debug, Default)]
pub struct Server(Option<SocketAddr>);

impl Server {
    pub fn bind(&mut self, ip: IpAddr, port: u16) {
        self.0 = Some(SocketAddr::new(ip, port))
    }
}

#[cfg(test)]
mod server_spec {
    use super::*;

    mod bind {
        use std::net::Ipv4Addr;

        use super::*;

        #[test]
        fn sets_provided_address_to_server() {
            let mut server = Server::default();

            server.bind(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
            assert_eq!(format!("{}", server.0.unwrap()), "127.0.0.1:8080");

            server.bind("::1".parse().unwrap(), 9911);
            assert_eq!(format!("{}", server.0.unwrap()), "[::1]:9911");
        }
    }
}
