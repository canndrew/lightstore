use super::*;
use std::io;
use failure::Fail;

#[derive(Debug)]
pub struct Peer {
    #[allow(unused)]
    stream: TcpStream,
    #[allow(unused)]
    sk: [u8; 32],
    #[allow(unused)]
    rk: [u8; 32],
    #[allow(unused)]
    sn: u64,
    #[allow(unused)]
    rn: u64,
}

impl Peer {
    pub fn connect(endpoint: &Endpoint, sec_key: &secp256k1::SecretKey)
        -> impl Future<Item = Peer, Error = ConnectError> + Send + 'static
    {
        let secp = Secp256k1::new();
        let ls_sk = sec_key.clone();
        let ls_pk = secp256k1::PublicKey::from_secret_key(&secp, sec_key);
        let remote_pub_key = endpoint.pub_key;
        TcpStream::connect(&endpoint.addr)
        .map_err(ConnectError::Connect)
        .and_then(move |stream| {
            Peer::initiate_handshake(secp, stream, ls_sk, ls_pk, remote_pub_key)
            .map_err(ConnectError::Handshake)
        })
    }

    fn initiate_handshake(
        secp: Secp256k1<secp256k1::All>,
        stream: TcpStream,
        ls_sk: secp256k1::SecretKey,
        ls_pk: secp256k1::PublicKey,
        rs: secp256k1::PublicKey,
    )
        -> impl Future<Item = Peer, Error = handshake::HandshakeError> + Send + 'static
    {
        let e_sk = secp256k1::SecretKey::new(&secp, &mut rand::thread_rng());
        let e_pk = secp256k1::PublicKey::from_secret_key(&secp, &e_sk);
        handshake::initiate_handshake(secp, stream, ls_sk, ls_pk, rs, e_sk, e_pk)
        .map(|(stream, sk, rk)| {
            Peer {
                stream, sk, rk,
                sn: 0,
                rn: 0,
            }
        })
    }

    pub fn send_msg(self, msg: Vec<u8>)
        -> impl Future<Item = Peer, Error = io::Error> + Send + 'static
    {
        let Peer { stream, sk, rk, sn, rn } = self;

        handshake::send_msg(stream, sk, msg, sn)
        .map(move |stream| {
            let sn = sn + 1;
            Peer {
                stream, sk, rk, sn, rn,
            }
        })
    }

    pub fn recv_msg(self)
        -> impl Future<Item = (Peer, Vec<u8>), Error = handshake::RecvMsgError> + Send + 'static
    {
        let Peer { stream, sk, rk, sn, rn } = self;

        handshake::recv_msg(stream, rk, rn)
        .map(move |(stream, msg)| {
            let rn = rn + 1;
            let peer = Peer {
                stream, sk, rk, sn, rn,
            };
            (peer, msg)
        })
    }
}

#[derive(Debug, Fail)]
pub enum ConnectError {
    #[fail(display = "tcp connect error: {}", _0)]
    Connect(io::Error),
    #[fail(display = "handshake failed: {}", _0)]
    Handshake(handshake::HandshakeError),
}

#[cfg(test)]
mod test {
    use super::*;
    use net_literals::*;
    use hex_literal::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_connect_to_real_network() {
        //use env_logger;
        //let _ = env_logger::init();

        let mut runtime = unwrap!(Runtime::new());
        let secp = Secp256k1::new();

        let endpoint = Endpoint {
            pub_key: unwrap!(secp256k1::PublicKey::from_slice(&secp, &hex!("02f6725f9c1c40333b67faea92fd211c183050f28df32cac3f9d69685fe9665432")[..])),
            addr: addr!("104.198.32.198:9735"),
        };

        let our_sk = secp256k1::SecretKey::new(&secp, &mut rand::thread_rng());
        let _our_pk = secp256k1::PublicKey::from_secret_key(&secp, &our_sk);

        runtime.block_on(future::lazy(move || {
            Peer::connect(&endpoint, &our_sk)
            .map_err(|e| panic!("error connecting: {}", e))
            .map(|_peer| {
                println!("connected!");
            })
        })).never_err()
    }
}

