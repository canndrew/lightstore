#![feature(never_type)]

use unwrap::unwrap;
use lightstore_units::{Btc, Sec, BtcPerSec};
use std::io;
use futures::sync::oneshot;
use futures::{Future, Stream, Async};
use future_utils::mpsc;
use future_utils::mpsc::{UnboundedSender, UnboundedReceiver};
use tokio::net::UdpSocket;
use std::collections::VecDeque;
use std::sync::Arc;
use std::mem;
use std::time::Instant;
use bytes::{Bytes, BytesMut};
use std::net::SocketAddr;
use canndrews_misc_ext_traits::{FutureExt, VecDequeExt};
use void::ResultVoidExt;

pub use self::outgoing_packet::OutgoingPacket;
use self::in_buffer::InBuffer;

mod outgoing_packet;
mod in_buffer;

#[cfg(test)]
mod test;

#[derive(Clone)]
pub struct SharedUdpSocket {
    packet_tx: UnboundedSender<Operation>,
}

struct SharedUdpSocketDriver {
    packet_rx: UnboundedReceiver<Operation>,
    out_queue: VecDeque<Pending>,
    socket: UdpSocket,
    in_buffer: InBuffer,
}

pub struct SendDgram {
    error_rx: oneshot::Receiver<Arc<io::Error>>,
}

pub struct RecvDgram {
    result_rx: oneshot::Receiver<io::Result<(BytesMut, SocketAddr)>>,
}

struct Pending {
    error_tx: oneshot::Sender<Arc<io::Error>>,
    packet: OutgoingPacket,
}

enum Operation {
    SendDgram(Pending),
    RecvDgram(oneshot::Sender<io::Result<(BytesMut, SocketAddr)>>),
}

impl SharedUdpSocket {
    pub fn share(socket: UdpSocket) -> SharedUdpSocket {
        let (out_packet_tx, out_packet_rx) = mpsc::unbounded();
        let driver = SharedUdpSocketDriver {
            socket,
            out_queue: VecDeque::new(),
            packet_rx: out_packet_rx,
            in_buffer: InBuffer::new(),
        };
        tokio::spawn(driver.infer_err());
        SharedUdpSocket {
            packet_tx: out_packet_tx,
        }
    }

    pub fn send_dgram(
        &mut self,
        packet: OutgoingPacket,
    ) -> SendDgram {
        let (send_dgram, pending) = SendDgram::new(packet);
        unwrap!(self.packet_tx.unbounded_send(Operation::SendDgram(pending)));
        send_dgram
    }

    pub fn recv_dgram(&mut self) -> RecvDgram {
        let (recv_dgram, result_tx) = RecvDgram::new();
        unwrap!(self.packet_tx.unbounded_send(Operation::RecvDgram(result_tx)));
        recv_dgram
    }
}

impl SendDgram {
    fn new(packet: OutgoingPacket) -> (SendDgram, Pending) {
        let (error_tx, error_rx) = oneshot::channel();
        let send_dgram = SendDgram { error_rx };
        let pending = Pending { error_tx, packet };
        (send_dgram, pending)
    }
}

impl RecvDgram {
    fn new() -> (RecvDgram, oneshot::Sender<io::Result<(BytesMut, SocketAddr)>>) {
        let (result_tx, result_rx) = oneshot::channel();
        let recv_dgram = RecvDgram { result_rx };
        (recv_dgram, result_tx)
    }
}

impl Future for SharedUdpSocketDriver {
    type Item = ();
    type Error = !;

    fn poll(&mut self) -> Result<Async<()>, !> {
        let shutting_down = loop {
            match self.packet_rx.poll().void_unwrap() {
                Async::Ready(Some(operation)) => {
                    match operation {
                        Operation::SendDgram(pending) => {
                            self.out_queue.push_back(pending);
                        },
                        Operation::RecvDgram(result_tx) => {
                            self.in_buffer.queue_receiver(result_tx);
                        },
                    }
                },
                Async::Ready(None) => break true,
                Async::NotReady => break false,
            }
        };

        let out_queue_empty = loop {
            let error_opt = match self.out_queue.front() {
                Some(pending) => {
                    let packet = &pending.packet;
                    match self.socket.poll_send_to(packet.as_bytes(), packet.dest()) {
                        Ok(Async::Ready(_n)) => None,
                        Ok(Async::NotReady) => break false,
                        Err(e) => Some(e),
                    }
                },
                None => break true,
            };
            match error_opt {
                Some(e) => {
                    let error = Arc::new(e);
                    let pendings = mem::replace(&mut self.out_queue, VecDeque::new());
                    for pending in pendings {
                        let _ = pending.error_tx.send(error.clone());
                    }
                },
                None => {
                    let _ = self.out_queue.pop_front();
                },
            }
        };

        let in_queue_empty = match self.in_buffer.poll_recv(&mut self.socket) {
            Async::Ready(()) => true,
            Async::NotReady => false,
        };

        let now = Instant::now();
        self.out_queue.insertion_sort_by(|pending0, pending1| {
            unwrap!(pending0.packet.utility_decay_at(now).partial_cmp(&pending1.packet.utility_decay_at(now)))
        });

        if shutting_down && out_queue_empty && in_queue_empty {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl Future for SendDgram {
    type Item = ();
    type Error = Arc<io::Error>;

    fn poll(&mut self) -> Result<Async<()>, Arc<io::Error>> {
        match self.error_rx.poll() {
            Ok(Async::Ready(error)) => Err(error),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(oneshot::Canceled) => Ok(Async::Ready(())),
        }
    }
}

impl Future for RecvDgram {
    type Item = (BytesMut, SocketAddr);
    type Error = Option<io::Error>;

    fn poll(&mut self) -> Result<Async<(BytesMut, SocketAddr)>, Option<io::Error>> {
        match self.result_rx.poll() {
            Ok(Async::Ready(Ok(x))) => Ok(Async::Ready(x)),
            Ok(Async::Ready(Err(e))) => Err(Some(e)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(oneshot::Canceled) => Err(None),
        }
    }
}

