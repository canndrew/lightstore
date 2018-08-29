use super::*;
//use crate::daemon::ai;
use futures::sync::oneshot;
use std::io;

#[cfg(test)]
use test;

pub struct Daemon {
    user_command_tx: UnboundedSender<UserCommand>,
}

pub struct Driver {
    peer_txs: HashMap<XorAddr, PeerTx>,
    user_command_rx: UnboundedReceiver<UserCommand>,
    socket: SharedUdpSocket,
}

enum UserCommand {
/*
    GetMutable {
        id: PublicSignKey,
        params: GetMutableParams,
        result_tx: oneshot::Sender<Vec<u8>>,
    },
*/
}

pub struct GetMutable;
impl Future for GetMutable {
    type Item = Bytes;
    type Error = !;
    fn poll(&mut self) -> Result<Async<Bytes>, !> {
        unimplemented!()
    }
}

impl Daemon {
    pub fn start() -> Result<(Daemon, SocketAddr), DaemonStartError> {
        unimplemented!()
        /*
        let (driver, addr, user_command_tx) = Driver::new()?;
        let daemon = Daemon {
            user_command_tx,
        };
        tokio::spawn(driver.infallible());
        Ok((daemon, addr))
        */
    }

    pub fn add_repo(&self, path: &Path) -> Result<git2::Repository, git2::Error> {
        git2::Repository::open(path)
    }

    pub fn get_mutable(
        &self,
        _id: PublicSignKey,
        _price: Btc,
        _price_decay_over_time: Sec,
        _price_decay_over_versions: f64,
    ) -> GetMutable {
        unimplemented!()
    /*
        let params = GetMutableParams {
            price,
            price_decay_over_time,
            price_decay_over_versions,
        };
        let (result_tx, result_rx) = oneshot::channel();
        let command = UserCommand::GetMutable {
            id,
            params,
            result_tx,
        };
        unwrap!(self.user_command_tx.unbounded_send(command));
        GetMutable {
            result_rx,
        }
    */
    }
}

impl Driver {
    fn new(bind_addr: &SocketAddr) -> Result<(Driver, UnboundedSender<UserCommand>), DaemonStartError> {
        let (user_command_tx, user_command_rx) = mpsc::unbounded();
        let socket = {
            UdpSocket::bind(bind_addr)
            .map_err(DaemonStartError::Bind)?
        };
        let socket = SharedUdpSocket::share(socket);
        let peer_txs = HashMap::new();
        let driver = Driver {
            peer_txs,
            user_command_rx,
            socket,
        };
        Ok((driver, user_command_tx))
    }

    /*
    fn default_network_state(id: XorAddr) -> BTreeMap<XorAddr, PeerInfo> {
        let id = id ^ {
            unwrap!(PublicSignKey::from_str("sdctcdd82z6yycjrv5yb2dax1m1294t4e4cm0c3d1ys1d8dv5zcg"))
            .to_xor_addr()
        };
        let node_info = PeerInfo {
            addr: Address::Domain(String::from("canndrew.org:45666")),
            exp_download_fee: LogBtcPerByte(0.0),
            var_download_fee: 0.0,
        };
        let mut ret = BTreeMap::new();
        ret.insert(id, node_info);
        ret
    }
    */
}

impl Future for Driver {
    type Item = ();
    type Error = Void;

    fn poll(&mut self) -> Result<Async<()>, Void> {
        /*
        if let Some(mut bootstrapping) = self.bootstrapping.take() {
            let bootstrapping = loop {
                match bootstrapping.poll().void_unwrap() {
                    Async::Ready(Some((pub_key, node_info))) => {
                        let key = self.sign_keypair.public.to_xor_addr() ^ pub_key.to_xor_addr();
                        let socket = self.socket.clone();
                        self.peers.insert(key, Peer::from_node_info(socket, node_info));
                    },
                    Async::Ready(None) => break None,
                    Async::NotReady => break Some(bootstrapping),
                }
            };
            self.bootstrapping = bootstrapping;
        }
        loop {
            let command = match self.user_command_rx.poll().void_unwrap() {
                Async::Ready(Some(command)) => command,
                Async::Ready(None) => return Ok(Async::Ready(())),
                Async::NotReady => break,
            };
            match command {
                UserCommand::GetMutable { id, params, result_tx } => {
                    let mut pending = self.pending_mutables.entry(id).or_default();
                    pending.add_client(params, result_tx);
                },
            }
        }
        /*
        loop {
            match unwrap!(self.incoming_endpoints.poll()) {
                Async::Ready(Some(udp_endpoint)) => {
                    let peer = Peer::from_endpoint(udp_endpoint);
                    self.peers.
                },
            }
        }
        */
        let node_key = self.sign_keypair.public;
        let peers = &mut self.peers;
        self.pending_mutables.retain(|key, pending_mutable| {
            match pending_mutable.poll(node_key, *key, peers) {
                Async::Ready(()) => false,
                Async::NotReady => true,
            }
        });
        for (_, peer) in peers {
            peer.poll(&self.socket);
        }
        */

        Ok(Async::NotReady)
    }
}

#[derive(Debug, Fail)]
pub enum DaemonStartError {
    #[fail(display = "error binding to udp socket: {}", _0)]
    Bind(io::Error),
}

