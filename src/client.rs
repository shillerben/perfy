use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

use crate::error;
use crate::netexp::{NetExp, NetExpParams, Side};

/// The Client connects to the Server, sends the NetExp to run,
/// and runs the NetExp when both the Client and Server are ready.
pub fn run(net_exp: NetExp) -> error::Result<()> {
    let client_params = match &net_exp {
        NetExp::Tcp(params) => params,
        NetExp::Udp(params) => params,
    };
    let server_params = NetExpParams {
        host: client_params.host,
        port: client_params.port,
        side: match client_params.side {
            Side::Rx => Side::Tx,
            Side::Tx => Side::Rx,
        },
        parallel: client_params.parallel,
        duration: client_params.duration,
    };
    let server_net_exp = match net_exp {
        NetExp::Tcp(_) => NetExp::Tcp(server_params),
        NetExp::Udp(_) => NetExp::Udp(server_params),
    };

    let mut stream = TcpStream::connect(format!("{}:{}", client_params.host, client_params.port))?;

    let mut buf = [0; 2];
    match client_params.side {
        Side::Tx => {
            // Send NetExp to Server then wait for Server to say that it's ready
            stream.write_all(&server_net_exp.serialize())?;
            stream.read_exact(&mut buf)?;
            if buf == "OK".as_bytes() {
                net_exp.run(|| {});
                Ok(())
            } else {
                Err(error::Error::new("Received invalid response from server"))
            }
        }
        Side::Rx => {
            // Set up listener before sending NetExp to Server
            let (ready_tx, ready_rx) = mpsc::channel::<()>();
            let exp_thread = thread::spawn(move || {
                net_exp.run(|| {
                    ready_tx.send(()).unwrap();
                })
            });
            let Ok(_) = ready_rx.recv_timeout(std::time::Duration::new(5, 0)) else {
                return Err(error::Error::new("Timed out initializing test"));
            };
            // Listener is ready, send NetExp to Server
            stream.write_all(&server_net_exp.serialize())?;
            stream.read_exact(&mut buf)?;
            if buf != "OK".as_bytes() {
                return Err(error::Error::new("Received invalid response from server"));
            }
            match exp_thread.join() {
                Err(_) => Err(error::Error::new("Failed joining thread")),
                Ok(_) => Ok(()),
            }
        }
    }
}
