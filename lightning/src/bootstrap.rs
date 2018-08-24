use super::*;

pub const DNS_SEEDS: &[&str] = &["lseed.bitcoinstats.com", "nodes.lightning.directory"];

pub fn bootstrap_lookup() -> impl Stream<Item = Endpoint, Error = BootstrapError> + Send {
    let secp = Secp256k1::new();

    let mut lookups = {
        stream::iter_ok(iter::repeat(DNS_SEEDS).flatten())
        .and_then(move |seed| {
            let seed = unwrap!(trust_dns_resolver::Name::from_str(seed));
            ResolverFuture::new(ResolverConfig::default(), ResolverOpts::default())
            .map_err(|e| BootstrapError::InitateResolver(Mutex::new(e)))
            .and_then(move |resolver| {
                resolver
                .lookup_srv(seed)
                .then(|res| Ok(res))
            })
        })
    };

    let mut lookup_errors = Vec::new();
    let lookups = stream::poll_fn(move || {
        loop {
            if lookup_errors.len() == DNS_SEEDS.len() {
                let lookup_errors = mem::replace(&mut lookup_errors, Vec::new());
                return Err(BootstrapError::AllSeedLookupsFailed(Mutex::new(lookup_errors)));
            }
            match lookups.poll()? {
                Async::Ready(Some(Ok(lookup))) => {
                    lookup_errors.clear();
                    return Ok(Async::Ready(Some(lookup)));
                },
                Async::Ready(Some(Err(lookup_error))) => {
                    lookup_errors.push(lookup_error);
                },
                Async::Ready(None) => unreachable!(),
                Async::NotReady => return Ok(Async::NotReady),
            }
        }
    });

    lookups
    .map(move |srv_lookup| {
        let mut rng = rand::thread_rng();
        let mut results = BTreeMap::new();
        let mut num_results = 0;
        for result in srv_lookup.iter() {
            num_results += 1;
            let result_vec: &mut Vec<_> = results.entry(result.priority()).or_default();
            result_vec.push(result);
        }

        let mut ordered_results = Vec::with_capacity(num_results);
        for (_, mut result_vec) in results {
            result_vec.sort_unstable_by(|result_0, result_1| {
                result_0.weight().cmp(&result_1.weight())
            });

            while !result_vec.is_empty() {
                let total_weight: u64 = result_vec.iter().map(|result| result.weight() as u64).sum();
                let target_weight = rng.gen_range(0, total_weight + 1);
                let mut sum_weight = 0;
                for i in 0..result_vec.len() {
                    sum_weight += result_vec[i].weight() as u64;
                    if target_weight <= sum_weight {
                        let result = result_vec.remove(i);
                        ordered_results.push((result.target().clone(), result.port()));
                        break;
                    }
                }
            }
        }

        stream::iter_ok(ordered_results)
    })
    .flatten()
    .filter_map(move |(name, port)| {
        let key = name.iter().next()?;
        let key = str::from_utf8(key).ok()?;
        let key = Bech32::from_str(key).ok()?;
        let key = bech32::convert_bits(key.data(), 5, 8, false).ok()?;
        let key = secp256k1::PublicKey::from_slice(&secp, &key).ok()?;
        Some((key, name, port))
    })
    .and_then(|(key, name, port)| {
        ResolverFuture::new(ResolverConfig::default(), ResolverOpts::default())
        .map_err(|e| BootstrapError::InitateResolver(Mutex::new(e)))
        .and_then(move |resolver| {
            resolver
            .lookup_ip(name)
            .then(move |ip_lookup_res| Ok((key, port, ip_lookup_res.ok())))
        })
    })
    .filter_map(|(key, port, ip_lookup_opt)| {
        let ip_lookup = ip_lookup_opt?;
        Some((key, port, ip_lookup))
    })
    .map(move |(key, port, ip_lookup)| {
        let ips: Vec<_> = ip_lookup.iter().collect();
        stream::iter_ok(ips)
        .map(move |ip| {
            Endpoint {
                pub_key: key.clone(),
                addr: SocketAddr::new(ip, port),
            }
        })
    })
    .flatten()
}

pub fn bootstrap(sec_key: &secp256k1::SecretKey) -> impl Stream<Item = Peer, Error = BootstrapError> {
    let sec_key = sec_key.clone();
    bootstrap_lookup()
    .and_then(move |endpoint| {
        Peer::connect(&endpoint, &sec_key)
        .then(|peer_res| Ok(peer_res.ok()))
    })
    .filter_map(|peer_opt| peer_opt)
}

#[derive(Debug, Fail)]
pub enum BootstrapError {
    #[fail(display = "error initiating DNS resolver: {:?}", _0)]
    InitateResolver(Mutex<ResolveError>),
    #[fail(display = "all DNS seed lookups failed with errors: {:?}", _0)]
    AllSeedLookupsFailed(Mutex<Vec<ResolveError>>),
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn bootstrap_to_network() {
        let mut runtime = unwrap!(Runtime::new());
        runtime.block_on(future::lazy(move || {
            let secp = Secp256k1::new();
            let our_sk = secp256k1::SecretKey::new(&secp, &mut rand::thread_rng());

            bootstrap(&our_sk)
            .into_future()
            .map_err(|(e, _bootstrap)| {
                panic!("bootstrap error: {}", e);
            })
            .map(|(peer_opt, _bootstrap)| {
                let peer = unwrap!(peer_opt);
                println!("got peer: {:?}", peer);
            })
        })).never_err()
    }
}

