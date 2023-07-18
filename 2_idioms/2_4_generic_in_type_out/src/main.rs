use std::{borrow::Cow, net::SocketAddr};

use default_struct_builder::DefaultBuilder;
use smart_default::SmartDefault;

fn main() {
    let _err = Error::default()
        .code("NO_USER")
        .status(404)
        .message("User not found");
}

#[derive(Debug, SmartDefault, DefaultBuilder)]
pub struct Error {
    #[builder(into)]
    #[default = "UNKNOWN"]
    code: Cow<'static, str>,
    #[default = 500]
    status: u16,
    #[builder(into)]
    #[default = "Unknown error has happened"]
    message: Cow<'static, str>,
}

#[derive(Debug, Default)]
pub struct Server(Option<SocketAddr>);

impl Server {
    pub fn bind(&mut self, addr: impl Into<SocketAddr>) {
        self.0 = Some(addr.into())
    }
}

#[cfg(test)]
mod server_spec {
    use super::*;

    mod bind {
        use std::net::{IpAddr, Ipv4Addr};

        use super::*;

        #[test]
        fn sets_provided_address_to_server() {
            let mut server = Server::default();

            server.bind((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
            assert_eq!(format!("{}", server.0.unwrap()), "127.0.0.1:8080");

            server.bind(("::1".parse::<IpAddr>().unwrap(), 9911));
            assert_eq!(format!("{}", server.0.unwrap()), "[::1]:9911");

            server.bind("127.0.0.1:8081".parse::<SocketAddr>().unwrap());
            assert_eq!(format!("{}", server.0.unwrap()), "127.0.0.1:8081");
        }
    }
}
