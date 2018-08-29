use super::*;
use git2::Repository;
use crate::crypto::ParseBase32Error;

use std::fs::File;
use std::{io, str};

pub trait RepositoryExt {
    fn create_lightstore_key(&self) -> Result<SignKeypair, CreateLightstoreKeyError>;
    fn add_lightstore_key(&self, keypair: &SignKeypair) -> Result<(), io::Error>;
    fn get_all_lightstore_keys(&self) -> Result<Vec<SignKeypair>, GetAllLightstoreKeysError>;
}

impl RepositoryExt for Repository {
    fn create_lightstore_key(&self) -> Result<SignKeypair, CreateLightstoreKeyError> {
        let keypair: SignKeypair = {
            SignKeypair::new()
            .map_err(CreateLightstoreKeyError::GenerateKey)?
        };
        self.add_lightstore_key(&keypair).map_err(CreateLightstoreKeyError::AddKey)?;
        Ok(keypair)
    }

    fn add_lightstore_key(&self, keypair: &SignKeypair) -> Result<(), io::Error> {
        let sk = InspectSecret(&keypair.secret).to_string();
        let pk = keypair.public.to_string();

        let mut path = self.path().to_owned();
        path.push("info");
        path.push("lightstore-keys");
        std::fs::create_dir_all(&path)?;

        path.push(pk);
        let mut f = File::create(&path)?;
        f.write_all(sk.as_bytes())?;

        Ok(())
    }

    fn get_all_lightstore_keys(&self) -> Result<Vec<SignKeypair>, GetAllLightstoreKeysError> {
        let mut path = self.path().to_owned();
        path.push("info");
        path.push("lightstore-keys");
        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            },
            Err(e) => return Err(GetAllLightstoreKeysError::Read(e)),
        };
        let mut ret = Vec::new();
        for entry in entries {
            let entry = entry.map_err(GetAllLightstoreKeysError::Read)?;
            let name = entry.file_name();
            let name = match name.to_str() {
                Some(name) => name,
                None => continue,
            };
            let pk = match PublicSignKey::from_str(&name) {
                Ok(pk) => pk,
                Err(..) => continue,
            };
            let mut path = path.clone();
            path.push(name);
            let mut file = File::open(&path).map_err(GetAllLightstoreKeysError::Read)?;
            let mut buffer = [0u8; 52];
            file.read_exact(&mut buffer[..]).map_err(GetAllLightstoreKeysError::Read)?;
            let sk = match str::from_utf8(&buffer[..]) {
                Ok(sk) => sk,
                Err(e) => return Err(GetAllLightstoreKeysError::InvalidKeyFile {
                    path,
                    kind: InvalidKeyFileError::InvalidUtf8(e),
                }),
            };
            let sk = match SecretSignKey::from_str(&sk) {
                Ok(sk) => sk,
                Err(e) => return Err(GetAllLightstoreKeysError::InvalidKeyFile {
                    path,
                    kind: InvalidKeyFileError::InvalidKey(e),
                }),
            };
            let keypair = SignKeypair {
                public: pk,
                secret: sk,
            };
            ret.push(keypair);
        }
        Ok(ret)
    }
}

#[derive(Debug, Fail)]
pub enum CreateLightstoreKeyError {
    #[fail(display = "error generating key: {}", _0)]
    GenerateKey(rand::Error),
    #[fail(display = "error adding key to git repo: {}", _0)]
    AddKey(io::Error),
}

#[derive(Debug, Fail)]
pub enum GetAllLightstoreKeysError {
    #[fail(display = "error reading lightstore-keys directory: {}", _0)]
    Read(io::Error),
    #[fail(display = "invalid key file {:?}: {}", path, kind)]
    InvalidKeyFile {
        path: PathBuf,
        kind: InvalidKeyFileError,
    },
}

#[derive(Debug, Fail)]
pub enum InvalidKeyFileError {
    #[fail(display = "{}", _0)]
    InvalidUtf8(std::str::Utf8Error),
    #[fail(display = "{}", _0)]
    InvalidKey(ParseBase32Error),
}


