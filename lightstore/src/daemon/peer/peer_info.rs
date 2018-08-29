use super::*;

#[derive(PartialEq)]
pub struct PeerInfo {
    pub addrs: Vec<Address>,
    pub exp_download_fee: LogBtcPerByte,
    pub var_download_fee: f64,
}

#[derive(PartialEq)]
pub struct Address {
    kind: AddressKind,
    probability: f64,
    probability_time: Instant,
    probability_decay: Sec,
}

#[derive(PartialEq)]
pub enum AddressKind {
    Resolved(SocketAddr),
    Domain(String),
}

impl PeerInfo {
    pub fn new() -> Arc<PeerInfo> {
        // TODO: pick proper values here
        let peer_info = PeerInfo {
            addrs: Vec::new(),
            exp_download_fee: resource_costs::download().log(),
            var_download_fee: 1.0,
        };
        Arc::new(peer_info)
    }

    pub fn from_msg(msg: &Msg) -> Arc<PeerInfo> {
        let peer_info = PeerInfo::new();
        peer_info.update(msg);
        peer_info
    }

    pub fn update(&self, _msg: &Msg) {
        unimplemented!()
    }
}

