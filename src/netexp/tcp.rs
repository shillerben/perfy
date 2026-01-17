use std::io::{Read, Write};
use std::net;
use std::time;

use super::NetExpParams;
use super::{BUF_SIZE, MB, Stats};
use crate::error;

/// Uninitialized
pub struct Uninit {}
/// Bound to port
pub struct Bound {
    listener: net::TcpListener,
}
/// Ready
pub struct Ready {
    stream: net::TcpStream,
}

pub struct TcpRx<State = Uninit> {
    params: NetExpParams,
    state: State,
}

impl TcpRx<Uninit> {
    pub fn new(params: NetExpParams) -> TcpRx<Uninit> {
        Self {
            params,
            state: Uninit {},
        }
    }

    pub fn bind(self) -> error::Result<TcpRx<Bound>> {
        let addr = format!("{}:{}", self.params.host, self.params.port);
        let listener = net::TcpListener::bind(addr.clone())?;
        println!("Started TcpRx listener on {}", addr);
        Ok(TcpRx {
            params: self.params,
            state: Bound { listener },
        })
    }
}

impl TcpRx<Bound> {
    pub fn accept(self) -> error::Result<TcpRx<Ready>> {
        let (stream, _) = self.state.listener.accept()?;
        Ok(TcpRx {
            params: self.params,
            state: Ready { stream },
        })
    }
}

impl TcpRx<Ready> {
    pub fn run(mut self) -> error::Result<Stats> {
        let mut buf: Vec<u8> = vec![0; BUF_SIZE];
        let peer_addr = self.state.stream.peer_addr()?;
        println!(
            "Running TCP recv {}:{} for {} seconds with {} threads...",
            peer_addr.ip(),
            peer_addr.port(),
            self.params.duration,
            self.params.parallel,
        );

        let start = time::Instant::now();

        for i in 0..self.params.duration {
            self.state.stream.read_exact(&mut buf)?;
            println!("{i} Received {BUF_SIZE} bytes");
        }

        let total_bytes = BUF_SIZE * self.params.duration as usize;
        let duration = time::Instant::now().duration_since(start).as_millis();
        let bandwidth = total_bytes as u128 / duration;

        Ok(Stats::new().with_bandwidth(bandwidth))
    }
}

pub struct TcpTx<State = Uninit> {
    params: NetExpParams,
    state: State,
}

impl TcpTx<Uninit> {
    pub fn new(params: NetExpParams) -> TcpTx<Uninit> {
        Self {
            params,
            state: Uninit {},
        }
    }

    pub fn init(self) -> error::Result<TcpTx<Ready>> {
        let addr = format!("{}:{}", self.params.host, self.params.port);
        println!("TcpTx connecting to {}", addr);
        let stream = net::TcpStream::connect(addr)?;
        Ok(TcpTx {
            params: self.params,
            state: Ready { stream },
        })
    }
}

impl TcpTx<Ready> {
    pub fn run(mut self) -> error::Result<Stats> {
        let mut buf: Vec<u8> = vec![0; BUF_SIZE];
        let peer_addr = self.state.stream.peer_addr()?;
        println!(
            "Running TCP send {}:{} for {} seconds with {} threads...",
            peer_addr.ip(),
            peer_addr.port(),
            self.params.duration,
            self.params.parallel,
        );

        let start = time::Instant::now();

        for i in 0..self.params.duration {
            buf[0] = i as u8;
            self.state.stream.write_all(&buf)?;
        }

        let total_bytes = BUF_SIZE * self.params.duration as usize;
        let duration_ms = time::Instant::now().duration_since(start).as_millis();
        let bandwidth = total_bytes as u128 / duration_ms;

        Ok(Stats::new().with_bandwidth(bandwidth))
    }
}
