use super::*;

pub struct InBuffer {
    buffer: BytesMut,
    result_txs: VecDeque<oneshot::Sender<io::Result<(BytesMut, SocketAddr)>>>,
}

impl InBuffer {
    pub fn new() -> InBuffer {
        InBuffer {
            buffer: BytesMut::new(),
            result_txs: VecDeque::new(),
        }
    }

    pub fn queue_receiver(&mut self, result_tx: oneshot::Sender<io::Result<(BytesMut, SocketAddr)>>) {
        self.result_txs.push_back(result_tx);
    }

    pub fn poll_recv(&mut self, socket: &mut UdpSocket) -> Async<()> {
        const MAX_UDP_SIZE: usize = 65535;
        loop {
            let result_tx = match self.result_txs.pop_front() {
                Some(result_tx) => result_tx,
                None => return Async::Ready(()),
            };
            self.buffer.reserve(MAX_UDP_SIZE);
            let res = unsafe {
                self.buffer.set_len(MAX_UDP_SIZE);
                let res = socket.poll_recv_from(&mut self.buffer[..]);
                let n = match res {
                    Ok(Async::Ready((n, _addr))) => n,
                    _ => 0,
                };
                self.buffer.set_len(n);
                res
            };
            match res {
                Ok(Async::Ready((_n, addr))) => {
                    let ret = self.buffer.take();
                    let _ = result_tx.send(Ok((ret, addr)));
                },
                Ok(Async::NotReady) => {
                    self.result_txs.push_front(result_tx);
                    let mut i = 0;
                    while i < self.result_txs.len() {
                        match unwrap!(self.result_txs[i].poll_cancel()) {
                            Async::Ready(()) => {
                                let _ = self.result_txs.swap_remove_back(i);
                            },
                            Async::NotReady => {
                                i += 1;
                            },
                        }
                    }
                    if !self.result_txs.is_empty() {
                        return Async::NotReady;
                    }
                },
                Err(e) => {
                    let _ = result_tx.send(Err(e));
                },
            }
        } 
    }
}

