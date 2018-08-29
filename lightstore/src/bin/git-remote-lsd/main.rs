#![feature(underscore_imports)]

use std::str;
use future_utils::{FutureExt, StreamExt, BoxSendFuture, BoxSendStream};
use git_remote_helper::{Ref, Object, PushObject};
use futures::Future;
#[allow(unused)]
use unwrap::*;
use std::io;
use lightstore::crypto::PublicSignKey;
use lightstore_units::*;
use futures::stream;
use std::path::Path;
use canndrews_misc_ext_traits::FutureExt as _;

struct App {
    _remote: String,
    key: PublicSignKey,
    _repo: git2::Repository,
    daemon: lightstore::Daemon,
}

impl git_remote_helper::RemoteHelper for App {
    type Fut = BoxSendFuture<App, io::Error>;

    fn new(remote: &str, url: &str, directory: &Path) -> BoxSendFuture<App, io::Error> {
        let remote = remote.to_owned();
        let directory = directory.to_owned();

        PublicSignKey::from_url(url)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))
        .map(move |key| {
            let (daemon, _addr) = unwrap!(lightstore::Daemon::start());
            let repo = unwrap!(daemon.add_repo(&directory));

            App {
                _remote: remote,
                key: key,
                _repo: repo,
                daemon,
            }
        })
        .into_send_boxed()
    }
}

impl git_remote_helper::List for App {
    type Items = BoxSendStream<Ref, io::Error>;

    fn list(&mut self) -> BoxSendStream<Ref, io::Error> {
        self.daemon
        .get_mutable(self.key, Btc(0.0), Sec(1.0), 1.0)
        .infer_err()
        .map(|bytes| {
            let refs = parse_refs(&bytes);
            stream::iter_ok(refs)
        })
        .flatten_stream()
        .into_send_boxed()
    }
}

impl git_remote_helper::ListForPush for App {
    fn list_for_push(&mut self) -> BoxSendStream<Ref, io::Error> {
        git_remote_helper::List::list(self)
    }
}

impl git_remote_helper::Push for App {
    type Fut = BoxSendFuture<(), io::Error>;

    fn push(&self, _objects: &[PushObject]) -> BoxSendFuture<(), io::Error> {
        panic!("push unimplemented");
    }
}

fn main() {
    git_remote_helper::run::<App>();
}

pub fn parse_refs(bytes: &[u8]) -> Vec<Ref> {
    let text = unwrap!(str::from_utf8(bytes));
    let mut ret = Vec::new();
    for line in text.lines() {
        let mut split = line.split_whitespace();
        let object = unwrap!(split.next());
        let object = if object.starts_with('@') {
            Object::Link(object[1..].to_string())
        } else {
            let mut v = Vec::new();
            unwrap!(base16::decode_buf(object, &mut v));
            let mut hash = [0u8; 20];
            hash[..].clone_from_slice(&v);
            Object::Hash(hash)
        };
        let name = unwrap!(split.next()).to_owned();
        let unchanged = match split.next() {
            Some("unchanged") => true,
            Some(..) => panic!("unknown attribute"),
            None => false,
        };
        match split.next() {
            Some(..) => panic!("unexpected arg"),
            None => (),
        }
        ret.push(Ref { object, name, unchanged });
    }
    ret
}

