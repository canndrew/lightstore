#![allow(unused_imports)] // because of https://github.com/rust-lang/rust/issues/45268

macro_rules! try_fut(
    ($e:expr) => (
        match $e {
            Ok(x) => x,
            Err(e) => return futures::future::err(e).into_send_boxed(),
        }
    )
);

macro_rules! slice_to_array(
    ($slice:expr, $len:expr) => ({
        union MaybeUninit<T: Copy> {
            init: T,
            uninit: (),
        }

        let mut array: MaybeUninit<[u8; $len]> = MaybeUninit { uninit: () };
        unsafe {
            array.init.copy_from_slice(&$slice[..]);
            array.init
        }
    })
);

mod bootstrap;
mod peer;
mod endpoint;
mod handshake;

pub use self::bootstrap::bootstrap;
pub use self::peer::*;
pub use self::endpoint::*;

use tokio::net::{TcpStream, TcpListener};
use futures::{future, stream, Future, Stream, Async};
use std::net::SocketAddr;
use std::str::FromStr;
use sha2::Sha256;
use std::sync::Mutex;
use std::{io, iter, mem, str};
use std::io::Cursor;
use std::collections::BTreeMap;
use hkdf::Hkdf;
use unwrap::unwrap;
use failure::Fail;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use secp256k1::Secp256k1;
use future_utils::FutureExt;
use std::hash::Hasher;
use digest::{Input, FixedOutput};
use canndrews_misc_ext_traits::ResultNeverErrExt;
use trust_dns_resolver::ResolverFuture;
use trust_dns_resolver::lookup::SrvLookup;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveError;
use rand::Rng;
use bech32::Bech32;

