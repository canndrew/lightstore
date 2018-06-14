use priv_prelude::*;
use tokio_io;

use bitcoin::{Msg, ReadMsgError, ParseVerAckError, ParseRejectError, Rejection};

pub struct Peer {
    #[allow(unused)]
    stream: TcpStream,
}

impl Peer {
    pub fn connect(handle: &Handle, addrs: impl ToSocketAddrs) -> impl Future<Item = Peer, Error = ConnectError> {
        let handle = handle.clone();
        addrs
        .to_socket_addrs()
        .into_future()
        .map_err(|e| {
            error!("error parsing addresses: {}", e);
            ConnectError::ParseAddr(e)
        })
        .and_then(move |addrs| {
            println!("got addresses");
            stream::iter_ok(addrs)
            .and_then(move |addr| {
                println!("got an address");
                TcpStream::connect(&addr, &handle)
                .map_err(|e| {
                    error!("error connecting: {}", e);
                    SingleConnectError::Connect(e)
                })
                .and_then(|stream| {
                    println!("connected");
                    let local_addr = unwrap!(stream.local_addr());
                    let remote_addr = unwrap!(stream.peer_addr());
                    let msg = Msg::version(&local_addr, &remote_addr);
                    tokio_io::io::write_all(stream, msg)
                    .map_err(|e| {
                        error!("error writing to socket: {}", e);
                        SingleConnectError::Io(e)
                    })
                    .and_then(|(stream, _msg)| {
                        println!("wrote message");
                        Msg::read_msg(stream)
                        .map_err(|e| {
                            error!("error reading response from socket: {}", e);
                            SingleConnectError::ReadMsg(e)
                        })
                        .and_then(|(stream, command, mut msg)| {
                            match &command[..] {
                                "verack" => {
                                    msg
                                    .parse_ver_ack()
                                    .map_err(|e| {
                                        error!("error parsing ver_ack message: {}", e);
                                        SingleConnectError::InvalidVerAck(e)
                                    })?;

                                    Ok(Peer {
                                        stream,
                                    })
                                }
                                "reject" => {
                                    let rejection = {
                                        msg
                                        .parse_reject()
                                        .map_err(|e| {
                                            error!("error parsing reject message: {}", e);
                                            SingleConnectError::ParseReject(e)
                                        })?
                                    };

                                    Err(SingleConnectError::ConnectionRejected(rejection))
                                }
                                _ => {
                                    Err(SingleConnectError::UnexpectedMessageKind(command))
                                }
                            }
                        })
                    })
                })
            })
            .first_ok()
            .map_err(ConnectError::Connect)
        })
    }
}

pub enum ConnectError {
    ParseAddr(io::Error),
    Connect(Vec<SingleConnectError>),
}

quick_error! {
    #[derive(Debug)]
    pub enum SingleConnectError {
        Connect(e: io::Error) {
            description("error connecting to address")
            display("error connecting to address: {}", e)
            cause(e)
        }
        Io(e: io::Error) {
            description("io error writing to socket")
            display("io error writing to socket: {}", e)
            cause(e)
        }
        InvalidVerAck(e: ParseVerAckError) {
            description("error parsing ver_ack response")
            display("error parsing ver_ack response: {}", e)
            cause(e)
        }
        ReadMsg(e: ReadMsgError) {
            description("error reading message from peer")
            display("error reading message from peer: {}", e)
            cause(e)
        }
        ParseReject(e: ParseRejectError) {
            description("error parsing reject message")
            display("error parsing reject message: {}", e)
            cause(e)
        }
        ConnectionRejected(e: Rejection) {
            description("connection rejected")
            display("connection rejected: {}", e)
        }
        UnexpectedMessageKind(command: String) {
            description("unexpected message kind")
            display("unexpected message kind: {}", command)
        }
    }
}

