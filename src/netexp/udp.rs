use std::net;

use super::NetExpParams;
use crate::error;

/// Uninitialized
pub struct Uninit {}
/// Ready
pub struct Ready {
    socket: net::UdpSocket,
}

pub struct UdpRx<State = Uninit> {
    params: NetExpParams,
    state: State,
}

impl UdpRx<Uninit> {
    pub fn new(params: NetExpParams) -> UdpRx<Uninit> {
        Self {
            params,
            state: Uninit {},
        }
    }

    pub fn bind(self) -> error::Result<UdpRx<Ready>> {
        let addr = format!("{}:{}", self.params.host, self.params.port);
        let socket = net::UdpSocket::bind(addr.clone())?;
        println!("Started UdpRx listener on {}", addr);
        Ok(UdpRx {
            params: self.params,
            state: Ready { socket },
        })
    }
}

impl UdpRx<Ready> {
    pub fn run(&self) -> error::Result<()> {
        let mut buf: Vec<u8> = vec![0; 1024];
        println!(
            "Running UDP recv {}:{} for {} seconds with {} threads...",
            self.params.host, self.params.port, self.params.duration, self.params.parallel,
        );

        let n_bytes = self.state.socket.recv(&mut buf)?;
        println!("Received {n_bytes} bytes");

        Ok(())
    }
}

pub struct UdpTx<State = Uninit> {
    params: NetExpParams,
    state: State,
}

impl UdpTx<Uninit> {
    pub fn new(params: NetExpParams) -> UdpTx<Uninit> {
        Self {
            params,
            state: Uninit {},
        }
    }

    pub fn init(self) -> error::Result<UdpTx<Ready>> {
        println!("UdpTx creating UDP socket");
        let socket = net::UdpSocket::bind(format!("{}:0", self.params.host))?;
        socket.connect(format!("{}:{}", self.params.host, self.params.port))?;
        Ok(UdpTx {
            params: self.params,
            state: Ready { socket },
        })
    }
}

impl UdpTx<Ready> {
    pub fn run(&self) -> error::Result<()> {
        println!(
            "Running UDP send {}:{} for {} seconds with {} threads...",
            self.params.host, self.params.port, self.params.duration, self.params.parallel,
        );

        self.state.socket.send("TESTING".as_bytes())?;

        Ok(())
    }
}
