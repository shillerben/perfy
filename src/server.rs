use std::net::IpAddr;

pub mod tcp_server;

pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

pub fn run(config: ServerConfig) {
    println!("server({}, {})", config.host, config.port);
}