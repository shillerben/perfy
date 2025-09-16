use std::net::IpAddr;

pub mod tcp_client;

pub struct ClientConfig {
    pub host: IpAddr,
    pub port: u16,
    pub udp: bool,
    pub parallel: u16,
    pub reverse: bool,
}

pub fn run(config: ClientConfig) {
    println!("client({}, {})", config.host, config.port);
}