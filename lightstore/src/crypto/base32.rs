use super::*;

pub fn as_base32(data: &[u8]) -> String {
    let mut s = base32_lib::encode(base32_lib::Alphabet::Crockford, data);
    s.make_ascii_lowercase();
    s
}

pub fn from_base32(s: &str, target: &mut [u8]) -> Result<(), ParseBase32Error> {
    let mut s = s.to_owned();
    s.make_ascii_uppercase();
    let b = {
        base32_lib::decode(base32_lib::Alphabet::Crockford, &s)
        .ok_or(ParseBase32Error::InvalidBase32)?
    };
    if b.len() != target.len() {
        return Err(ParseBase32Error::InvalidLen);
    }
    target[..].clone_from_slice(&b);
    Ok(())
}

#[derive(Debug, Fail)]
pub enum ParseBase32Error {
    #[fail(display = "invalid base32")]
    InvalidBase32,
    #[fail(display = "invalid length")]
    InvalidLen,
}

