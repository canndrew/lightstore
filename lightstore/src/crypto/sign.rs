use super::*;
use trust_dns_resolver::ResolverFuture;
use sha2::Sha512;

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PublicSignKey {
    bytes: [u8; 32],
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct SecretSignKey {
    bytes: Secure<[u8; 32]>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct SignKeypair {
    pub public: PublicSignKey,
    pub secret: SecretSignKey,
}

impl PublicSignKey {
    pub fn to_xor_addr(&self) -> XorAddr {
        XorAddr::from_bytes(self.as_bytes())
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        self.bytes
    }

    pub fn from_bytes(bytes: [u8; 32]) -> PublicSignKey {
        PublicSignKey { bytes }
    }

    pub fn to_url(&self) -> String {
        format!("lsd://{}/", self)
    }

    pub fn from_url(url: &str) -> impl Future<Item = PublicSignKey, Error = FromUrlError> {
        let parsed_url = match Url::parse(url) {
            Ok(parsed_url) => parsed_url,
            Err(e) => return {
                future::err(FromUrlError::MalformedUrl(MalformedUrlError::Parse(e)))
                .into_send_boxed()
            },
        };
        let host = match parsed_url.host_str() {
            Some(host) => host,
            None => return {
                future::err(FromUrlError::MalformedUrl(MalformedUrlError::MissingHost))
                .into_send_boxed()
            },
        };
        if let Ok(key) = PublicSignKey::from_str(host) {
            return future::ok(key).into_send_boxed();
        }
        let host = host.to_owned();
        let url_path = parsed_url.path().to_owned();

        ResolverFuture::from_system_conf()
        .into_future()
        .flatten()
        .map_err(|e| FromUrlError::Resolve(e.to_string()))
        .and_then(move |resolver| {
            resolver
            .txt_lookup(host)
            .map_err(|e| FromUrlError::Resolve(e.to_string()))
            .and_then(move |txt_lookup| {
                for txt in txt_lookup.iter() {
                    for line in txt.iter() {
                        let line = match str::from_utf8(&**line) {
                            Ok(line) => line,
                            Err(..) => continue,
                        };
                        let mut split = line.split_whitespace();
                        match split.next() {
                            Some("lightstore") => (),
                            _ => continue,
                        };
                        let key = match split.next() {
                            Some(key) => key,
                            None => continue,
                        };
                        let path = match split.next() {
                            Some(path) => path,
                            None => continue,
                        };
                        match split.next() {
                            Some(..) => continue,
                            None => (),
                        }
                        if path != url_path {
                            continue;
                        }
                        let key = match PublicSignKey::from_str(key) {
                            Ok(key) => key,
                            Err(e) => return Err(FromUrlError::InvalidKey(e)),
                        };
                        return Ok(key);
                    }
                }
                Err(FromUrlError::MissingKey)
            })
        })
        .into_send_boxed()
    }
}

impl SecretSignKey {
    pub fn from_bytes(bytes: &mut [u8; 32]) -> SecretSignKey {
        SecretSignKey { bytes: Secure::move_from(bytes) }
    }
}

impl SignKeypair {
    pub fn new() -> Result<SignKeypair, rand::Error> {
        let mut rng = OsRng::new()?;
        let keypair = ed25519_dalek::Keypair::generate::<Sha512, _>(&mut rng);
        let public = PublicSignKey::from_bytes(keypair.public.to_bytes());
        let secret = SecretSignKey::from_bytes(&mut keypair.secret.to_bytes());
        Ok(SignKeypair { public, secret })
    }
}

impl fmt::Debug for SecretSignKey {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut hasher = DefaultHasher::new();
        hasher.write(&self.bytes[..]);
        fmt.debug_tuple("SecretSignKey").field(&hasher.finish()).finish()
    }
}

impl fmt::Display for PublicSignKey {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = base32::as_base32(&self.bytes[..]);
        write!(fmt, "{}", s)
    }
}

impl fmt::Debug for PublicSignKey {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("PublicSignKey").field(self).finish()
    }
}

impl FromStr for PublicSignKey {
    type Err = ParseBase32Error;

    fn from_str(s: &str) -> Result<PublicSignKey, ParseBase32Error> {
        let mut bytes = [0u8; 32];
        base32::from_base32(s, &mut bytes[..])?;
        Ok(PublicSignKey { bytes })
    }
}

impl FromStr for SecretSignKey {
    type Err = ParseBase32Error;

    fn from_str(s: &str) -> Result<SecretSignKey, ParseBase32Error> {
        let mut bytes = [0u8; 32];
        base32::from_base32(s, &mut bytes[..])?;
        Ok(SecretSignKey::from_bytes(&mut bytes))
    }
}

impl<'a> fmt::Debug for InspectSecret<'a, SecretSignKey> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("SecretSignKey").field(self).finish()
    }
}

impl<'a> fmt::Display for InspectSecret<'a, SecretSignKey> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = base32::as_base32(&self.0.bytes[..]);
        write!(fmt, "{}", s)
    }
}

#[derive(Debug, Fail)]
pub enum FromUrlError {
    #[fail(display = "malformed url ({})", _0)]
    MalformedUrl(MalformedUrlError),
    #[fail(display = "{}", _0)]
    Resolve(String),
    #[fail(display = "no key under that path found in host's DNS record")]
    MissingKey,
    #[fail(display = "invalid key specified in DNS record: {}", _0)]
    InvalidKey(ParseBase32Error),
}

#[derive(Debug, Fail)]
pub enum MalformedUrlError {
    #[fail(display = "{}", _0)]
    Parse(url::ParseError),
    #[fail(display = "no hostname specified")]
    MissingHost,
}

