mod tcp;
mod udp;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use bytes::{Bytes, BytesMut, Buf, BufMut};

use crate::error;

#[derive(Clone, Debug, PartialEq)]
pub struct NetExpParams {
    pub host: IpAddr,
    pub port: u16,
    pub side: Side,
    pub parallel: u16,
    pub duration: u16,
}

#[derive(Debug, PartialEq)]
pub enum NetExp {
    Tcp(NetExpParams),
    Udp(NetExpParams),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    Tx,
    Rx,
}

impl NetExp {
    pub fn run<F>(&self, ready_cb: F)
        where F: FnOnce()
    {
        match self {
            NetExp::Tcp(params) => {
                match params.side {
                    Side::Rx => {
                        let rx = tcp::TcpRx::new(params.clone());
                        let rx = rx.bind().unwrap();
                        ready_cb();
                        let rx = rx.accept().unwrap();
                        rx.run().unwrap();
                    },
                    Side::Tx => {
                        let tx = tcp::TcpTx::new(params.clone());
                        let tx = tx.init().unwrap();
                        ready_cb();
                        tx.run().unwrap();
                    },
                }
            }
            NetExp::Udp(params) => {
                match params.side {
                    Side::Rx => {
                        let rx = udp::UdpRx::new(params.clone());
                        let rx = rx.bind().unwrap();
                        ready_cb();
                        rx.run().unwrap();
                    },
                    Side::Tx => {
                        let tx = udp::UdpTx::new(params.clone());
                        let tx = tx.init().unwrap();
                        ready_cb();
                        tx.run().unwrap();
                    },
                }
            }
        }
    }

    pub fn serialize(&self) -> Bytes {
        // 1 byte for NetExp variant
        // 1 byte for IpV4/IpV6
        // 16 bytes for IpAddr
        // 2 bytes for port
        // 1 byte for side
        // 2 bytes for parallel
        // 2 bytes for duration
        let mut bytes = BytesMut::with_capacity(25);
        let params = match self {
            NetExp::Tcp(params) => {
                bytes.put_u8(0);
                params
            },
            NetExp::Udp(params) => {
                bytes.put_u8(1);
                params
            }
        };
        match params.host {
            IpAddr::V4(ipv4addr) => {
                bytes.put_u8(0);
                bytes.put_bytes(0, 12);
                for byte in ipv4addr.octets() {
                    bytes.put_u8(byte);
                }
            },
            IpAddr::V6(ipv6addr) => {
                bytes.put_u8(1);
                for byte in ipv6addr.octets() {
                    bytes.put_u8(byte);
                }
            }
        }
        bytes.put_u16(params.port);
        match params.side {
            Side::Rx => bytes.put_u8(0),
            Side::Tx => bytes.put_u8(1),
        }
        bytes.put_u16(params.parallel);
        bytes.put_u16(params.duration);

        bytes.freeze()
    }

    pub fn deserialize(mut bytes: &[u8]) -> error::Result<Self> {
        // first byte tells us which enum variant to use
        let variant = bytes.get_u8();
        let host = match bytes.get_u8() {
            0 => {
                bytes.advance(12);
                let host = bytes.get_u32();
                IpAddr::V4(Ipv4Addr::from_bits(host))
            },
            1 => {
                let host = bytes.get_u128();
                IpAddr::V6(Ipv6Addr::from_bits(host))
            },
            _ => {
                return Err(error::Error::new("Invalid host"))
            },
        };
        let port = bytes.get_u16();
        let side = match bytes.get_u8() {
            0 => Side::Rx,
            1 => Side::Tx,
            _ => {
                return Err(error::Error::new("Invalid side"))
            },
        };
        let parallel = bytes.get_u16();
        let duration = bytes.get_u16();
        let params = NetExpParams {
            host, port, side, parallel, duration
        };
        match variant {
            0 => Ok(NetExp::Tcp(params)),
            1 => Ok(NetExp::Udp(params)),
            _ => Err(error::Error::new("Invalid NetExp"))
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_serialize_and_deserialize_tcp_ipv4_rx() {
        let in_bytes: Bytes = vec![
            0, // TCP
            0, // IPv4
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 127, 0, 0, 1, // host + padding
            0, 80, // port
            0, // Rx
            0, 4, // parallel
            0, 30, // duration
        ].into();
        let expected = NetExp::Tcp(NetExpParams {
            host: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
            port: 80,
            side: Side::Rx,
            parallel: 4,
            duration: 30,
        });
        let net_exp = NetExp::deserialize(&in_bytes).expect("Failed to deserialize NetExp");
        assert_eq!(net_exp, expected);
        let out_bytes = net_exp.serialize();
        assert_eq!(out_bytes, in_bytes);
    }

    #[test]
    fn test_serialize_and_deserialize_udp_ipv6_tx() {
        let in_bytes: Bytes = vec![
            1, // UDP
            1, // IPv6
            0, 1, 0, 2, 0, 3, 0, 4,
            1, 0, 2, 0, 3, 0, 4, 0, // host + padding
            1, 2, // port 258
            1, // Tx
            1, 4, // parallel
            0, 30, // duration
        ].into();
        let expected = NetExp::Udp(NetExpParams {
            host: IpAddr::V6(Ipv6Addr::from_str("1:2:3:4:100:200:300:400").unwrap()),
            port: 0x0102,
            side: Side::Tx,
            parallel: 0x0104,
            duration: 30,
        });
        let net_exp = NetExp::deserialize(&in_bytes).expect("Failed to deserialize NetExp");
        assert_eq!(net_exp, expected);
        let out_bytes = net_exp.serialize();
        assert_eq!(out_bytes, in_bytes);
    }

    #[test]
    #[should_panic]
    fn test_bad_deserialize_variant() {
        let in_bytes: Bytes = vec![
            200, // BAD!!!
            0, // IPv4
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 127, 0, 0, 1, // host + padding
            0, 80, // port
            0, // Rx
            0, 4, // parallel
            0, 30, // duration
        ].into();
        NetExp::deserialize(&in_bytes).expect("Failed to deserialize NetExp");
    }

    #[test]
    #[should_panic]
    fn test_bad_deserialize_proto() {
        let in_bytes: Bytes = vec![
            0, // TCP
            10, // BAD!!!
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 127, 0, 0, 1, // host + padding
            0, 80, // port
            0, // Rx
            0, 4, // parallel
            0, 30, // duration
        ].into();
        NetExp::deserialize(&in_bytes).expect("Failed to deserialize NetExp");
    }

    #[test]
    #[should_panic]
    fn test_bad_deserialize_host() {
        let in_bytes: Bytes = vec![
            0, // TCP
            0, // IPv4
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 127, 0, 0, // NOT ENOUGH BYTES
            0, 80, // port
            0, // Rx
            0, 4, // parallel
            0, 30, // duration
        ].into();
        NetExp::deserialize(&in_bytes).expect("Failed to deserialize NetExp");
    }

    #[test]
    #[should_panic]
    fn test_bad_deserialize_side() {
        let in_bytes: Bytes = vec![
            0, // TCP
            0, // IPv4
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 127, 0, 0, 1, // host + padding
            0, 80, // port
            10, // BAD!!!
            0, 4, // parallel
            0, 30, // duration
        ].into();
        NetExp::deserialize(&in_bytes).expect("Failed to deserialize NetExp");
    }
}