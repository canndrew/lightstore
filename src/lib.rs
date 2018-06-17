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
#[cfg_attr(test, macro_use)]
extern crate net_literals;
//#[macro_use]
#[cfg_attr(test, macro_use)]
extern crate hex_literal;
extern crate secp256k1;
//extern crate rand_0_3;
extern crate hkdf;
extern crate chacha20_poly1305_aead;

macro_rules! try_fut(
    ($e:expr) => (
        match $e {
            Ok(x) => x,
            Err(e) => return future::err(e).into_boxed(),
        }
    )
);

macro_rules! slice_to_array(
    ($slice:expr, $len:expr) => ({
        let mut array: MaybeUninit<[u8; $len]> = MaybeUninit { uninit: () };
        unsafe {
            array.init.copy_from_slice(&$slice[..]);
            array.init
        }
    })
);

mod priv_prelude;
pub mod bitcoin;
pub mod lightning;
mod ext;
mod util;

