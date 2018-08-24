use super::*;
use futures::{future, stream};
use futures::future::Loop;
use futures::stream::FuturesUnordered;
use future_utils::FutureExt;
use tokio::runtime::Runtime;
use canndrews_misc_ext_traits::{ResultNeverErrExt, BytesMutExt};
use net_literals::*;

fn send_data(num_clients: usize, message_size: usize) -> impl Future<Item = (), Error = !> {
    future::lazy(move || {
        const DATA: usize = 1000;

        let send_socket = unwrap!(UdpSocket::bind(&addr!("0.0.0.0:0")));
        let recv_socket = unwrap!(UdpSocket::bind(&addr!("0.0.0.0:0")));

        let addr = unwrap!(recv_socket.local_addr());

        let send_socket = SharedUdpSocket::share(send_socket);
        let recv_socket = SharedUdpSocket::share(recv_socket);

        let data_per_client = DATA / num_clients;
        let num_messages = data_per_client / message_size;
        let mut senders = FuturesUnordered::new();
        let mut receivers = FuturesUnordered::new();
        let (send_message_tx, send_message_rx) = mpsc::unbounded();
        let (recv_message_tx, recv_message_rx) = mpsc::unbounded();
        for _ in 0..num_clients {
            let mut messages = Vec::with_capacity(num_messages);
            for _ in 0..num_messages {
                let message = BytesMut::random(message_size).freeze();
                messages.push(message);
            }
            let mut send_socket = send_socket.clone();
            let send_message_tx = send_message_tx.clone();
            let sender = {
                stream::iter_ok(messages)
                .for_each(move |message| {
                    unwrap!(send_message_tx.unbounded_send(message.clone()));
                    let packet = OutgoingPacket::new(message, addr, Btc(1.0), Sec(1.0));
                    send_socket.send_dgram(packet)
                })
            };
            senders.push(sender);

            let mut recv_socket = recv_socket.clone();
            let recv_message_tx = recv_message_tx.clone();
            let receiver = future::loop_fn((), move |()| {
                let recv_message_tx = recv_message_tx.clone();
                recv_socket
                .recv_dgram()
                .map_err(|e| panic!("error receiving: {:?}", e))
                .and_then(move |(message, _addr)| {
                    unwrap!(recv_message_tx.unbounded_send(message));
                    Ok(Loop::Continue(()))
                })
            });
            receivers.push(receiver);
        }
        drop(send_socket);
        drop(recv_socket);
        drop(send_message_tx);
        drop(recv_message_tx);

        let senders = senders.for_each(|()| Ok(()));
        let receivers = receivers.for_each(|()| Ok(()));

        let send_all = {
            send_message_rx
            .collect()
            .while_driving(senders)
            .map(|(messages, _)| messages)
            .map_err(|(v, _)| void::unreachable(v))
        };

        let recv_all = {
            recv_message_rx
            .take(num_messages as u64)
            .collect()
            .while_driving(receivers)
            .map(|(messages, _)| messages)
            .map_err(|(v, _)| void::unreachable(v))
        };

        send_all
        .join(recv_all)
        .map(move |(mut send_messages, mut recv_messages)| {
            let mut dropped = 0;
            send_messages.sort_unstable();
            recv_messages.sort_unstable();
            println!("num sent == {}", send_messages.len());
            println!("num recv == {}", recv_messages.len());
            while let Some(message) = recv_messages.pop() {
                loop {
                    if unwrap!(send_messages.pop()) == message {
                        break;
                    }
                    dropped += 1;
                }
            }
            assert!(dropped < num_messages / 2);
        })
    })
}

#[ignore]
#[test]
fn send_data_one_client_small_messages() {
    let mut runtime = unwrap!(Runtime::new());
    runtime.block_on(send_data(1, 16)).never_err()
}

#[ignore]
#[test]
fn send_data_two_clients_small_messages() {
    let mut runtime = unwrap!(Runtime::new());
    runtime.block_on(send_data(2, 16)).never_err()
}

#[ignore]
#[test]
fn send_data_twenty_clients_small_messages() {
    let mut runtime = unwrap!(Runtime::new());
    runtime.block_on(send_data(20, 16)).never_err()
}

