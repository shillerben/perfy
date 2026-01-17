use clap::{Args, Parser, Subcommand};
use std::net::IpAddr;

use perfy::netexp;
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
    Client(ClientArgs),
}

#[derive(Args)]
struct ClientArgs {
    #[command(subcommand)]
    command: ClientCommands,
}

#[derive(Args)]
struct CommonClientArgs {
    /// server host to connect to
    #[arg(short = 'c', long = "host")]
    host: String,
    /// server port to connect to
    #[arg(short = 'p', long = "port")]
    port: u16,
    /// number of parallel streams
    #[arg(short = 'P', long = "parallel", default_value_t = 1)]
    parallel: u16,
    /// number of seconds to run for
    #[arg(short = 't', long = "time", default_value_t = 10)]
    duration: u16,
    /// send data from server to client instead of client to server
    #[arg(short = 'R', long = "reverse", default_value_t = false)]
    reverse: bool,
}

#[derive(Subcommand)]
enum ClientCommands {
    /// test using TCP
    Tcp(CommonClientArgs),

    /// test using UDP
    Udp(CommonClientArgs),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Server { host, port } => {
            let host: IpAddr = host.parse().expect("Invalid host");
            let config = server::ServerConfig { host, port };
            server::run(config).unwrap_or_else(|e| print_error_and_exit(&e.message))
        }
        Commands::Client(client_args) => match client_args.command {
            ClientCommands::Tcp(args) => {
                let host: IpAddr = args.host.parse().expect("Invalid host");
                let params = netexp::NetExpParams {
                    host,
                    port: args.port,
                    side: if args.reverse {
                        netexp::Side::Rx
                    } else {
                        netexp::Side::Tx
                    },
                    parallel: args.parallel,
                    duration: args.duration,
                };
                let net_exp = netexp::NetExp::Tcp(params);
                client::run(net_exp).unwrap_or_else(|e| print_error_and_exit(&e.message))
            }
            ClientCommands::Udp(args) => {
                let host: IpAddr = args.host.parse().expect("Invalid host");
                let params = netexp::NetExpParams {
                    host,
                    port: args.port,
                    side: if args.reverse {
                        netexp::Side::Rx
                    } else {
                        netexp::Side::Tx
                    },
                    parallel: args.parallel,
                    duration: args.duration,
                };
                let net_exp = netexp::NetExp::Udp(params);
                client::run(net_exp).unwrap_or_else(|e| print_error_and_exit(&e.message))
            }
        },
    }
}

fn print_error_and_exit(s: &str) -> ! {
    eprintln!("{}", s);
    std::process::exit(1);
}
