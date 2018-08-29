use super::*;
use generic_array::typenum::U32;
use failure::Fail;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SignPublicKey {
    bytes: [u8; 32],
}

#[derive(PartialEq, Eq, Clone)]
pub struct SignSecretKey {
    bytes: Secure<U32>,
}

#[derive(PartialEq, Eq, Clone)]
pub struct SignKeyPair {
    pub public: SignPublicKey,
    pub secret: SignSecretKey,
}

#[derive(Debug, Fail)]
#[fail(display = "verification error")]
pub struct VerificationError;

impl SignKeyPair {
    pub fn new() -> SignKeyPair {
        let mut public = SignPublicKey {
            bytes: [0u8; 32],
        };
        let secret = SignSecretKey {
            bytes: Secure::new(|bytes| unsafe {
                libsodium_sys::crypto_sign_keypair(public.bytes.as_mut_ptr(), bytes.as_mut_ptr());
            }),
        };
        SignKeyPair { public, secret }
    }
}

impl SignPublicKey {
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), VerificationError> {
        let mut padded_signature = [0u8; 64];
        padded_signature[..signature.len()].clone_from_slice(signature);
        let res = unsafe {
            libsodium_sys::crypto_sign_verify_detached(
                signature.as_ptr(),
                message.as_ptr(),
                message.len() as u64,
                self.bytes.as_ptr(),
            )
        };
        match res {
            0 => Ok(()),
            -1 => Err(VerificationError),
            _ => unreachable!(),
        }
    }
}

impl SignSecretKey {
    pub fn sign(&self, message: &[u8], signature: &mut BytesMut) {
        let signature_reserve = 64;
        let signature_original_len = signature.len();
        let mut signature_used = 0;
        signature.reserve(signature_reserve);
        let bytes_ref = self.bytes.get_ref();
        let res = unsafe {
            libsodium_sys::crypto_sign_detached(
                signature.as_mut_ptr().offset(signature_original_len as isize),
                &mut signature_used,
                message.as_ptr(),
                message.len() as u64,
                bytes_ref.as_ptr(),
            )
        };
        debug_assert_eq!(res, 0);
        unsafe {
            signature.set_len(signature_original_len + signature_used as usize);
        }
    }
}

