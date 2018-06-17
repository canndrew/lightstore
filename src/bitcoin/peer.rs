use priv_prelude::*;
use tokio_io;

use bitcoin::{Msg, ReadMsgError, ParseVerAckError, ParseRejectError, Rejection, ParseVersionError};

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
            stream::iter_ok(addrs)
            .and_then(move |addr| {
                TcpStream::connect(&addr, &handle)
                .map_err(|e| {
                    error!("error connecting: {}", e);
                    SingleConnectError::Connect(e)
                })
                .and_then(|stream| {
                    Peer::version_handshake(stream)
                })
            })
            .first_ok()
            .map_err(ConnectError::Connect)
        })
    }

    fn version_handshake(stream: TcpStream) -> impl Future<Item = Peer, Error = SingleConnectError> {
        let local_addr = unwrap!(stream.local_addr());
        let remote_addr = unwrap!(stream.peer_addr());
        let msg = Msg::version(&local_addr, &remote_addr);
        tokio_io::io::write_all(stream, msg)
        .map_err(|e| {
            error!("error writing to socket: {}", e);
            SingleConnectError::Io(e)
        })
        .and_then(|(stream, _msg)| {
            Msg::read_msg(stream)
            .map_err(|e| {
                error!("error reading response from socket: {}", e);
                SingleConnectError::ReadMsg(e)
            })
            .and_then(|(stream, command, mut msg)| {
                match &command[..] {
                    "reject" => {
                        let rejection = try_fut!(
                            msg
                            .parse_reject()
                            .map_err(|e| {
                                error!("error parsing reject message: {}", e);
                                SingleConnectError::ParseReject(e)
                            })
                        );
                        
                        error!("rejected by peer: {}", rejection);
                        return {
                            future::err(SingleConnectError::ConnectionRejected(rejection))
                            .into_boxed()
                        };
                    }
                    "version" => {
                        try_fut!(
                            msg
                            .parse_version()
                            .map_err(|e| {
                                error!("error parsing version message: {}", e);
                                SingleConnectError::ParseVersion(e)
                            })
                        );
                    },
                    _ => {
                        error!("unexpected message kind: {}", command);
                        return {
                            future::err(SingleConnectError::UnexpectedMessageKind(command))
                            .into_boxed()
                        };
                    }
                };

                Msg::read_msg(stream)
                .map_err(|e| {
                    error!("error reading response from socket: {}", e);
                    SingleConnectError::ReadMsg(e)
                })
                .and_then(|(stream, command, msg)| {
                    match &command[..] {
                        "verack" => {
                            try_fut!(
                                msg
                                .parse_ver_ack()
                                .map_err(|e| {
                                    error!("error parsing ver_ack message: {}", e);
                                    SingleConnectError::InvalidVerAck(e)
                                })
                            );
                        }
                        _ => {
                            error!("unexpected message kind: {}", command);
                            return {
                                future::err(SingleConnectError::UnexpectedMessageKind(command))
                                .into_boxed()
                            };
                        }
                    };
                    future::ok(Peer {
                        stream,
                    })
                    .into_boxed()
                })
                .into_boxed()
            })
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
        ParseVersion(e: ParseVersionError) {
            description("error parsing version message")
            display("error parsing version message: {}", e)
            cause(e)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    //use env_logger;

    #[test]
    fn connect_to_mainnet() {
        //let _ = env_logger::init();

        let mut core = unwrap!(Core::new());
        let handle = core.handle();
        let f = {
            Peer::connect(&handle, "seed.btc.petertodd.org:8333")
            .map_err(|e| panic!(e))
            .map(|_peer| {
                println!("connected!");
            })
        };
        core.run(f).void_unwrap()
    }
}

