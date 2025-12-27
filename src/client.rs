use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};
use std::thread;
use std::sync::mpsc;

use crate::error;
use crate::netexp::{NetExp, NetExpParams, Side};

pub struct ClientConfig {
    pub host: IpAddr,
    pub port: u16,
    pub udp: bool,
    pub parallel: u16,
    pub duration: u16,
    pub reverse: bool,
}

pub fn run(config: ClientConfig) -> error::Result<()> {
    let mut buf = vec![0; 1024];
    println!("client({}, {})", config.host, config.port);
    let params = NetExpParams {
        host: config.host,
        port: config.port,
        side: if config.reverse { Side::Rx } else { Side::Tx },
        parallel: config.parallel,
        duration: config.duration,
    };
    let server_params = NetExpParams {
        host: config.host,
        port: config.port,
        side: match params.side {
            Side::Rx => Side::Tx,
            Side::Tx => Side::Rx,
        },
        parallel: config.parallel,
        duration: config.duration,
    };
    let net_exp = if config.udp {
        NetExp::Udp(params)
    } else {
        NetExp::Tcp(params)
    };
    let server_net_exp = if config.udp {
        NetExp::Udp(server_params)
    } else {
        NetExp::Tcp(server_params)
    };

    let mut stream = TcpStream::connect(format!("{}:{}", config.host, config.port))?;

    if config.reverse {
        let (ready_tx, ready_rx) = mpsc::channel::<()>();
        let exp_thread = thread::spawn(move || {
            net_exp.run(|| { ready_tx.send(()).unwrap(); })
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
    } else {
        stream.write_all(&server_net_exp.serialize())?;
        stream.read_exact(&mut buf[..2])?;
        if &buf[..2] == "OK".as_bytes() {
            net_exp.run(|| {});
            Ok(())
        } else {
            Err(error::Error::new("Received invalid response from server"))
        }
    }
}