use priv_prelude::*;
use lightning::handshake;
use secp256k1;
use rand;

use lightning::Endpoint;

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
    pub fn connect(endpoint: &Endpoint, sec_key: &secp256k1::SecretKey, handle: &Handle)
        -> impl Future<Item = Peer, Error = ConnectError>
    {
        let secp = Secp256k1::new();
        let ls_sk = sec_key.clone();
        let ls_pk = unwrap!(secp256k1::PublicKey::from_secret_key(&secp, sec_key));
        let remote_pub_key = endpoint.pub_key;
        TcpStream::connect(&endpoint.addr, handle)
        .map_err(ConnectError::Connect)
        .and_then(move |stream| {
            Peer::initiate_handshake(secp, stream, ls_sk, ls_pk, remote_pub_key)
            .map_err(ConnectError::Handshake)
        })
    }

    fn initiate_handshake(
        secp: Secp256k1,
        stream: TcpStream,
        ls_sk: secp256k1::SecretKey,
        ls_pk: secp256k1::PublicKey,
        rs: secp256k1::PublicKey,
    )
        -> impl Future<Item = Peer, Error = handshake::HandshakeError>
    {
        let (e_sk, e_pk) = unwrap!(secp.generate_keypair(&mut rand::thread_rng()));
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
        -> impl Future<Item = Peer, Error = io::Error>
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
        -> impl Future<Item = (Peer, Vec<u8>), Error = handshake::RecvMsgError>
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

quick_error! {
    #[derive(Debug)]
    pub enum ConnectError {
        Connect(e: io::Error) {
            description("tcp connect")
            display("tcp connect: {}", e)
            cause(e)
        }
        Handshake(e: handshake::HandshakeError) {
            description("handshake failed")
            display("handshake failed: {}", e)
            cause(e)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_connect_to_real_network() {
        //use env_logger;
        //let _ = env_logger::init();

        let mut core = unwrap!(Core::new());
        let handle = core.handle();

        let secp = Secp256k1::new();
        let endpoint = Endpoint {
            pub_key: unwrap!(secp256k1::PublicKey::from_slice(&secp, &hex!("02f6725f9c1c40333b67faea92fd211c183050f28df32cac3f9d69685fe9665432")[..])),
            addr: addr!("104.198.32.198:9735"),
        };

        let (our_sk, _our_pk) = unwrap!(secp.generate_keypair(&mut rand::thread_rng()));

        core.run(
            Peer::connect(&endpoint, &our_sk, &handle)
            .map_err(|e| panic!("error connecting: {}", e))
            .map(|_peer| {
                println!("connected!");
            })
        ).void_unwrap()
    }
}

