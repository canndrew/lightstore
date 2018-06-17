use priv_prelude::*;
use secp256k1;

#[derive(Clone, Copy)]
pub struct Endpoint {
    pub pub_key: secp256k1::PublicKey,
    pub addr: SocketAddr,
}

