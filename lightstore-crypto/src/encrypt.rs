use super::*;
use generic_array::typenum::U32;
use failure::Fail;

/// A public key, used for authenticated encryption.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct PublicKey {
    bytes: [u8; 32],
}

/// A secret key, used for authenticated encryption.
#[derive(PartialEq, Eq, Clone)]
pub struct SecretKey {
    bytes: Secure<U32>,
}

/// A shared secret key, used for authenticated encryption.
#[derive(PartialEq, Eq, Clone)]
pub struct SharedKey {
    bytes: Secure<U32>,
}

/// A pair of keys used for authenticated encryption.
#[derive(PartialEq, Eq, Clone)]
pub struct KeyPair {
    pub public: PublicKey,
    pub secret: SecretKey,
}

#[derive(Debug, Fail)]
#[fail(display = "decryption error")]
pub struct DecryptError;

impl KeyPair {
    pub fn new() -> KeyPair {
        let mut public = PublicKey {
            bytes: [0u8; 32],
        };
        let secret = SecretKey {
            bytes: Secure::new(|bytes| unsafe {
                libsodium_sys::crypto_box_keypair(public.bytes.as_mut_ptr(), bytes.as_mut_ptr());
            }),
        };
        KeyPair { public, secret }
    }
}

impl PublicKey {
    pub fn encrypt(&self, plaintext: &[u8], ciphertext: &mut BytesMut) {
        let ciphertext_reserve = plaintext.len() + 48;
        ciphertext.reserve(ciphertext_reserve);
        let ciphertext_original_len = ciphertext.len();
        let res = unsafe {
            libsodium_sys::crypto_box_seal(
                ciphertext.as_mut_ptr().offset(ciphertext_original_len as isize),
                plaintext.as_ptr(),
                plaintext.len() as u64,
                self.bytes.as_ptr(),
            )
        };
        debug_assert_eq!(res, 0);
        unsafe {
            ciphertext.set_len(ciphertext_original_len + ciphertext_reserve);
        }
    }
}

impl SecretKey {
    pub fn decrypt(&self, ciphertext: &[u8], plaintext: &mut BytesMut) -> Result<(), DecryptError> {
        let plaintext_reserve = ciphertext.len() - 48;
        plaintext.reserve(plaintext_reserve);
        let plaintext_original_len = plaintext.len();
        let public_key = self.public_key();
        let bytes_ref = self.bytes.get_ref();
        let res = unsafe {
            libsodium_sys::crypto_box_seal_open(
                plaintext.as_mut_ptr().offset(plaintext_original_len as isize),
                ciphertext.as_ptr(),
                ciphertext.len() as u64,
                public_key.bytes.as_ptr(),
                bytes_ref.as_ptr(),
            )
        };
        match res {
            0 => {
                unsafe {
                    plaintext.set_len(plaintext_original_len + plaintext_reserve);
                };
                Ok(())
            },
            -1 => Err(DecryptError),
            _ => unreachable!(),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        let mut public = PublicKey {
            bytes: [0u8; 32],
        };
        let bytes_ref = self.bytes.get_ref();
        unsafe {
            libsodium_sys::crypto_scalarmult_base(public.bytes.as_mut_ptr(), bytes_ref.as_ptr());
        }
        public
    }
    
    pub fn shared_key(&self, public: &PublicKey) -> SharedKey {
        let bytes_ref = self.bytes.get_ref();
        SharedKey {
            bytes: Secure::new(|bytes| unsafe {
                let res = libsodium_sys::crypto_box_beforenm(
                    bytes.as_mut_ptr(),
                    public.bytes.as_ptr(),
                    bytes_ref.as_ptr(),
                );
                debug_assert!(res == 0);
            }),
        }
    }
}

impl SharedKey {
    pub fn encrypt(&self, nonce: [u8; 24], message: &mut [u8], mac: &mut [u8; 16]) {
        let bytes_ref = self.bytes.get_ref();
        let res = unsafe {
            libsodium_sys::crypto_box_detached_afternm(
                message.as_mut_ptr(),
                mac.as_mut_ptr(),
                message.as_ptr(),
                message.len() as u64,
                nonce.as_ptr(),
                bytes_ref.as_ptr(),
            )
        };
        debug_assert_eq!(res, 0);
    }

    pub fn decrypt(&self, nonce: [u8; 24], message: &mut [u8], mac: &[u8; 16]) -> Result<(), DecryptError> {
        let bytes_ref = self.bytes.get_ref();
        let res = unsafe {
            libsodium_sys::crypto_box_open_detached_afternm(
                message.as_mut_ptr(),
                message.as_ptr(),
                mac.as_ptr(),
                message.len() as u64,
                nonce.as_ptr(),
                bytes_ref.as_ptr(),
            )
        };
        match res {
            0 => Ok(()),
            -1 => Err(DecryptError),
            _ => unreachable!(),
        }
    }
}

