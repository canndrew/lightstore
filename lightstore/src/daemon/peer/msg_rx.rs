pub struct MsgRx {
    socket: SharedUdpSocket,
    state: MsgRxState,
    peer_db: Arc<PeerDb>,
}

enum MsgRxState {
    Invalid,
    Waiting,
    Receiving(lightstore_shared_udp_socket::RecvDgram),
    Unpacking(Cursor<Bytes>),
}

impl MsgRx {
    pub fn new(socket: SharedUdpSocket, peer_db: Arc<PeerDb>) -> MsgRx {
        MsgRx {
            socket,
            peer_db,
            packet: None,
        }
    }
}

impl Stream for MsgRx {
    type Item = Msg;
    type Error = io::Error;

    fn poll(&mut self) -> io::Result<Async<Option<Msg>>> {
        loop {
            let state = mem::replace(&mut self.state, MsgRxState::Invalid);
            match state {
                MsgRxState::Invalid => unreachable!(),
                MsgRxState::Receiving(mut recv_dgram) => {
                    match recv_dgram.poll()? {
                        Async::Ready((data, addr)) => {
                            let bytes = Cursor::new(data.freeze());
                            self.state = MsgRxState::Unpacking(bytes);
                        }, 
                        Async::NotReady => {
                            self.state = MsgRxState::Receiving(recv_dgram);
                            return Ok(Async::NotReady);
                        },
                    }
                },
                MsgRxState::Unpacking(mut bytes) => {
                    if bytes.remaining() == 0 {
                        self.state = MsgRxState::Receiving(self.socket.recv_dgram());
                        continue;
                    }

                    let msg_res = Msg::read(&mut bytes);
                    self.state = MsgRxState::Unpacking(bytes);
                    Ok(Async::Ready(Some((msg_res, addr))))
                },
            }
        }
    }
}

