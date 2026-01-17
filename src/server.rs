use std::io::{Read, Write};
use std::net::IpAddr;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

use crate::error;
use crate::netexp::NetExp;

pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

/// The Server receives NetExp from the Client, sets up the Rx side of the
/// NetExp if necessary, then sends "OK" to the Client.
pub fn run(config: ServerConfig) -> error::Result<()> {
    loop {
        let Ok(listener) = TcpListener::bind(format!("{}:{}", config.host, config.port)) else {
            return Err(error::Error::new(&format!(
                "Failed binding to {}:{}",
                config.host, config.port
            )));
        };

        println!("listening on {}:{}", config.host, config.port);

        let Ok((stream, _)) = listener.accept() else {
            return Err(error::Error::new("Error accepting client connection"));
        };
        // stop listening in case a Tcp test needs to rebind to the port
        drop(listener);
        handle_client(stream).unwrap_or_else(|e| eprintln!("Error handling client {}", e))
    }
}

/// Deserialize NetExp from Client and run NetExp
fn handle_client(mut stream: TcpStream) -> error::Result<()> {
    let client_addr = stream.peer_addr()?;
    println!("Got a client! {}", client_addr);
    let mut buf = [0; NetExp::serialized_size()];
    stream.read_exact(&mut buf)?;

    let mut experiment = NetExp::deserialize(&buf)?;
    match &mut experiment {
        NetExp::Tcp(params) => {
            params.host = client_addr.ip();
        }
        NetExp::Udp(params) => {
            params.host = client_addr.ip();
        }
    };

    let (ready_tx, ready_rx) = mpsc::channel::<()>();
    let exp_thread = thread::spawn(move || {
        experiment.run(|| {
            ready_tx.send(()).unwrap();
        })
    });
    let Ok(_) = ready_rx.recv_timeout(std::time::Duration::new(5, 0)) else {
        return Err(error::Error::new("Timed out initializing test"));
    };

    let response = "OK".as_bytes();
    stream.write_all(response)?;
    println!("Sent response!");

    match exp_thread.join() {
        Err(_) => Err(error::Error::new("Failed joining thread")),
        Ok(_) => Ok(()),
    }
}
