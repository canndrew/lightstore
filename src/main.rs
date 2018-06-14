#![feature(proc_macro, generators)]

extern crate tokio_core;
extern crate tokio_io;
//extern crate futures;
#[macro_use]
extern crate unwrap;
extern crate future_utils;
extern crate void;
extern crate futures_await as futures;
extern crate byteorder;
extern crate bytes;
extern crate sha2;
extern crate rand;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
extern crate env_logger;

macro_rules! try_fut(
    ($e:expr) => (
        match $e {
            Ok(x) => x,
            Err(e) => return future::err(e).into_boxed(),
        }
    )
);

mod priv_prelude;
mod bitcoin;
mod ext;

use priv_prelude::*;
use bitcoin::Peer;

fn main() {
    let _ = env_logger::init();

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

