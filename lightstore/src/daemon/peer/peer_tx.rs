use super::*;
use std::io;
use trust_dns_resolver::ResolverFuture;
use lightstore_shared_udp_socket::{SendDgram, OutgoingPacket};

const MAX_MSG_LEN: usize = 500;

pub struct PeerTx {
    message_tx: UnboundedSender<PendingSendMessage>,
}

struct PeerDriver {
    message_rx: UnboundedReceiver<PendingSendMessage>,
    send_messages: VecDeque<PendingSendMessage>,
    socket: SharedUdpSocket,
    sending: Option<(SendDgram, Vec<oneshot::Sender<Result<(), PeerSendError>>>)>,
    out_buffer: BytesMut,
    peer_info: Arc<PeerInfo>,
}

pub enum PeerSendError {
    Socket(Arc<io::Error>),
    TooExpensive,
}

pub struct SendMessage {
    result_rx: oneshot::Receiver<Result<(), PeerSendError>>,
}

struct PendingSendMessage {
    outgoing_msg: OutgoingMsg,
    result_tx: oneshot::Sender<Result<(), PeerSendError>>,
}

impl PeerTx {
    pub fn from_peer_info(socket: SharedUdpSocket, peer_info: Arc<PeerInfo>) -> PeerTx {
        let (message_tx, message_rx) = mpsc::unbounded();
        let peer_driver = PeerDriver {
            message_rx,
            send_messages: VecDeque::new(),
            socket,
            sending: None,
            out_buffer: BytesMut::new(),
            peer_info,
        };
        tokio::spawn(peer_driver.infer_err());
        let peer_tx = PeerTx {
            message_tx,
        };
        peer_tx
    }

    pub fn send_message(
        &self,
        outgoing_msg: OutgoingMsg,
    ) -> SendMessage {
        let (result_tx, result_rx) = oneshot::channel();
        let pending = PendingSendMessage { outgoing_msg, result_tx };
        unwrap!(self.message_tx.unbounded_send(pending));
        SendMessage {
            result_rx,
        }
    }

    pub fn update_info(&self, _info: Arc<PeerInfo>) {
        unimplemented!()
    }
}

impl PeerDriver {
    fn create_packet(&mut self) -> (OutgoingPacket, Vec<oneshot::Sender<Result<(), PeerSendError>>>) {
        //let mut rng = rand::thread_rng();
        let mtu = MAX_MSG_LEN;
        self.out_buffer.reserve(mtu);

        let msg = unwrap!(self.send_messages.pop_front());
        msg.outgoing_msg.msg.write(&mut self.out_buffer);
        let bytes = self.out_buffer.take().freeze();
        let sending = vec![msg.result_tx];
        let packet = OutgoingPacket {
            data: bytes,
            dest: addr!("0.0.0.0:0"),
            utility: msg.outgoing_msg.utility,
            utility_time: msg.outgoing_msg.utility_time,
            utility_decay: msg.outgoing_msg.utility_decay,
        };
        (packet, sending)
    }
}

impl Future for PeerDriver {
    type Item = ();
    type Error = !;

    fn poll(&mut self) -> Result<Async<()>, !> {
        let shutting_down = loop {
            match self.message_rx.poll().void_unwrap() {
                Async::Ready(Some(send_message)) => self.send_messages.push_back(send_message),
                Async::Ready(None) => break true,
                Async::NotReady => break false,
            }
        };

        let queue_empty = loop {
            let sending = self.sending.take();
            if let Some((mut sending, result_txs)) = sending {
                match sending.poll() {
                    Ok(Async::Ready(())) => {
                        for result_tx in result_txs {
                            let _ = result_tx.send(Ok(()));
                        }
                    },
                    Ok(Async::NotReady) => {
                        self.sending = Some((sending, result_txs));
                        break false;
                    },
                    Err(e) => {
                        for result_tx in result_txs {
                            let _ = result_tx.send(Err(PeerSendError::Socket(e.clone())));
                        }
                    },
                }
            }

            let now = Instant::now();
            self.send_messages.insertion_sort_by(|send_msg_0, send_msg_1| {
                let decay_0 = send_msg_0.outgoing_msg.utility_decay_at(now);
                let decay_1 = send_msg_1.outgoing_msg.utility_decay_at(now);
                unwrap!(decay_0.partial_cmp(&decay_1))
            });

            if self.send_messages.is_empty() {
                break true;
            } {
                let (packet, send_messages) = self.create_packet();
                let sending = self.socket.send_dgram(packet);
                self.sending = Some((sending, send_messages));
            }
        };

        if shutting_down && queue_empty {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

