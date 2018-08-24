use super::*;

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub pub_key: secp256k1::PublicKey,
    pub addr: SocketAddr,
}

