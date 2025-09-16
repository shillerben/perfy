use std::net::IpAddr;
use clap::{Parser, Subcommand};

use perfy::{client, server};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// run perfy server
    Server {
        /// interface to bind to
        #[arg(short = 'B', long = "bind")]
        host: String,
        /// port to bind to
        #[arg(short = 'p', long = "port")]
        port: u16,
    },
    /// run perfy client
    Client {
        /// server host to connect to
        #[arg(short = 'c', long = "host")]
        host: String,
        /// server port to connect to
        #[arg(short = 'p', long = "port")]
        port: u16,
        /// use UDP instead of TCP
        #[arg(short = 'u', long = "udp", default_value_t = false)]
        udp: bool,
        /// number of parallel streams
        #[arg(short = 'P', long = "parallel", default_value_t = 1)]
        parallel: u16,
        /// send data from server to client instead of client to server
        #[arg(short = 'R', long = "reverse", default_value_t = false)]
        reverse: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Server { host, port } => {
            let host: IpAddr = host.parse().expect("invalid host");
            let config = server::ServerConfig { host, port };
            server::run(config);
        },
        Commands::Client { host, port, udp, parallel, reverse } => {
            let host: IpAddr = host.parse().expect("invalid host");
            let config = client::ClientConfig { host, port, udp, parallel, reverse };
            client::run(config);
        },
    }
}