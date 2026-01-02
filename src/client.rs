use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

use crate::error;
use crate::netexp::{NetExp, NetExpParams, Side};

pub fn run(net_exp: NetExp) -> error::Result<()> {
    let mut buf = vec![0; 1024];
    let params = match &net_exp {
        NetExp::Tcp(params) => params,
        NetExp::Udp(params) => params,
    };
    println!("client({}, {})", params.host, params.port);
    let server_params = NetExpParams {
        host: params.host,
        port: params.port,
        side: match params.side {
            Side::Rx => Side::Tx,
            Side::Tx => Side::Rx,
        },
        parallel: params.parallel,
        duration: params.duration,
    };
    let server_net_exp = match net_exp {
        NetExp::Tcp(_) => NetExp::Tcp(server_params),
        NetExp::Udp(_) => NetExp::Udp(server_params),
    };

    let mut stream = TcpStream::connect(format!("{}:{}", params.host, params.port))?;

    match params.side {
        Side::Tx => {
            stream.write_all(&server_net_exp.serialize())?;
            stream.read_exact(&mut buf[..2])?;
            if &buf[..2] == "OK".as_bytes() {
                net_exp.run(|| {});
                Ok(())
            } else {
                Err(error::Error::new("Received invalid response from server"))
            }
        }
        Side::Rx => {
            let (ready_tx, ready_rx) = mpsc::channel::<()>();
            let exp_thread = thread::spawn(move || {
                net_exp.run(|| {
                    ready_tx.send(()).unwrap();
                })
            });
            let Ok(_) = ready_rx.recv_timeout(std::time::Duration::new(5, 0)) else {
                return Err(error::Error::new("Timed out initializing test"));
            };
            stream.write_all(&server_net_exp.serialize())?;
            stream.read_exact(&mut buf[..2])?;
            if &buf[..2] != "OK".as_bytes() {
                return Err(error::Error::new("Received invalid response from server"));
            }
            match exp_thread.join() {
                Err(_) => Err(error::Error::new("Failed joining thread")),
                Ok(_) => Ok(()),
            }
        }
    }
}
