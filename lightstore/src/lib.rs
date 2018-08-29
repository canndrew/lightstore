#![feature(never_type)]
#![feature(underscore_imports)]
#![allow(unused_imports)]

#[macro_export]
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

pub mod git;
pub mod daemon;
//pub mod priv_prelude;
pub mod crypto;
pub mod resource_costs;

pub use crate::daemon::Daemon;
use std::path::{Path, PathBuf};
use std::{str, fmt, mem, ptr};
use std::io::{Read, Write, Cursor};
use std::ops::Deref;
use unwrap::*;
use failure::Fail;
use url::Url;
use futures::{future, Future, Stream, Sink, Async, IntoFuture};
use futures::sync::oneshot;
use future_utils::{FutureExt, BoxSendFuture, BoxSendStream};
use future_utils::mpsc::{self, UnboundedSender, UnboundedReceiver};
use void::{ResultVoidExt, Void};
use std::net::SocketAddr;
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{atomic, Arc};
use std::str::FromStr;
use self::crypto::*;
use tokio::net::UdpSocket;
use tokio::timer::Delay;
use net_literals::*;
use std::time::{Duration, Instant};
use bytes::{Bytes, BytesMut, Buf, BufMut, IntoBuf};
use std::marker::PhantomData;
use lightstore_shared_udp_socket::SharedUdpSocket;
use atomic_arc::AtomicArc;
use lightstore_units::*;
use canndrews_misc_ext_traits::{FutureExt as _, VecDequeExt};
use rand::OsRng;

