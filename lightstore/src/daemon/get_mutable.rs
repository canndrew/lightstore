use super::*;
use crate::daemon::{Peer, Msg};
use futures::sync::oneshot;

pub struct GetMutable {
    crate result_rx: oneshot::Receiver<Vec<u8>>,
}

impl Future for GetMutable {
    type Item = Vec<u8>;
    type Error = Void;

    fn poll(&mut self) -> Result<Async<Vec<u8>>, Void> {
        Ok(unwrap!(self.result_rx.poll()))
    }
}

pub struct GetMutableParams {
    pub price: Btc,
    pub price_decay_over_time: Sec,
    pub price_decay_over_versions: f64,
}

#[derive(Default)]
pub struct PendingGetMutable {
    clients: Vec<PendingGetMutableClient>,
    peers_messaged: BTreeMap<PublicEncryptKey, Instant>,
}

pub struct PendingGetMutableClient {
    result_tx: oneshot::Sender<Vec<u8>>,
    params: GetMutableParams,
}

impl PendingGetMutable {
    pub fn add_client(
        &mut self,
        params: GetMutableParams,
        result_tx: oneshot::Sender<Vec<u8>>,
    ) {
        self.clients.push(PendingGetMutableClient {
            params, result_tx,
        })
    }

    pub fn poll(
        &mut self,
        node_id: PublicSignKey,
        data_id: PublicSignKey,
        known_peers: &mut BTreeMap<XorAddr, Peer>,
    ) -> Async<()>
    {
        if !self.peers_messaged.is_empty() {
            return Async::NotReady;
        }

        let key = node_id.to_xor_addr() ^ data_id.to_xor_addr();

        // TODO: pick proper params
        let params = GetMutableParams {
            price: Btc(0.0),
            price_decay_over_time: Sec(1.0),
            price_decay_over_versions: 1.0,
        };
        let msg = Msg::SenderGetMutable {
            id: data_id,
            params: params,
        };
        for (_, peer) in known_peers.range_mut(&key..) {
            peer.send_message(msg);
            return Async::NotReady;
        }
        for (_, peer) in known_peers.range_mut(..&key) {
            peer.send_message(msg);
            return Async::NotReady;
        }
        Async::NotReady
    }
}

