use super::*;

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PublicKey {
    bytes: [u8; 32],
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct SecretKey {
    bytes: Secure<[u8; 32]>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct SharedKey {
    bytes: Secure<[u8; 32]>,
}

impl PublicKey {
    pub fn as_bytes(&self) -> [u8; 32] {
        self.bytes
    }

    pub fn from_bytes(bytes: [u8; 32]) -> PublicKey {
        PublicKey { bytes }
    }
}

impl SecretKey {
    pub fn new() -> Result<SecretKey, rand::Error> {
        let mut rng = OsRng::new()?;
        let mut bytes = x25519_dalek::generate_secret(&mut rng);
        let bytes = Secure::move_from(&mut bytes);
        Ok(SecretKey { bytes })
    }

    pub fn to_public_key(&self) -> PublicKey {
        let point = x25519_dalek::generate_public(&self.bytes);
        let bytes = point.to_bytes();
        PublicKey { bytes }
    }

    pub fn from_bytes(bytes: &mut [u8; 32]) -> SecretKey {
        SecretKey { bytes: Secure::move_from(bytes) }
    }

    pub fn create_shared_key(&self, public_key: &PublicKey) -> SharedKey {
        let mut bytes = x25519_dalek::diffie_hellman(&self.bytes, &public_key.bytes);
        let bytes = Secure::move_from(&mut bytes);
        SharedKey { bytes }
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut hasher = DefaultHasher::new();
        hasher.write(&self.bytes[..]);
        fmt.debug_tuple("SecretKey").field(&hasher.finish()).finish()
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = base32::as_base32(&self.as_bytes());
        write!(fmt, "{}", s)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("PublicKey").field(self).finish()
    }
}

impl FromStr for PublicKey {
    type Err = ParseBase32Error;

    fn from_str(s: &str) -> Result<PublicKey, ParseBase32Error> {
        let mut bytes = [0u8; 32];
        base32::from_base32(s, &mut bytes[..])?;
        Ok(PublicKey { bytes })
    }
}

impl FromStr for SecretKey {
    type Err = ParseBase32Error;

    fn from_str(s: &str) -> Result<SecretKey, ParseBase32Error> {
        let mut bytes = [0u8; 32];
        base32::from_base32(s, &mut bytes[..])?;
        Ok(SecretKey::from_bytes(&mut bytes))
    }
}

impl<'a> fmt::Debug for InspectSecret<'a, SecretKey> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("SecretKey").field(self).finish()
    }
}

impl<'a> fmt::Display for InspectSecret<'a, SecretKey> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = base32::as_base32(&self.0.bytes[..]);
        write!(fmt, "{}", s)
    }
}

