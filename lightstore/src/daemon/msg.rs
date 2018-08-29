use super::*;
//use crate::daemon::GetMutableParams;

pub enum Msg {
    SenderDownloadFee {
        btc_per_byte: BtcPerByte,
    },
    /*
    SenderGetMutable {
        id: PublicSignKey,
        params: GetMutableParams,
    },
    */
    /*
    SenderGetAddress {
        reward: Btc,
        reward_decay_time: Sec,
    },
    SenderSignKey {
        sign_key: PublicSignKey,
    },
    SenderEncryptKey {
        encrypt_key: PublicEncryptKey,
    },
    MutableData {
        signature: PublicSignKey,
        data: ContentData,
    },
    ObjectData {
        object_hash: ObjectHash,
        data: ContentData,
    },
    MerkleData {
        data: Vec<u8>,
    },
    */
}

mod tag {
    pub const SENDER_DOWNLOAD_FEE: u16 = 0;
    pub const SENDER_GET_MUTABLE: u16 = 1;
}

pub struct OutgoingMsg {
    pub msg: Msg,
    pub utility: Btc,
    pub utility_time: Instant,
    pub utility_decay: Sec,
}

/*
pub enum ContentData {
    Data(Vec<u8>),
    Hash {
        hash_depth: NonZeroU8,
        hash: MerkleHash,
    },
}
*/

impl Msg {
    pub fn write(&self, _bytes: &mut BytesMut) {
        match self {
            Msg::SenderDownloadFee { .. } => unimplemented!(),
            /*
            Msg::SenderGetMutable { id, params } => {
                bytes.put_u16_be(tag::SENDER_GET_MUTABLE);
                bytes.put_slice(&id.as_bytes());
                bytes.put_f64_be(params.price.val());
                bytes.put_f64_be(params.price_decay_over_time.val());
                bytes.put_f64_be(params.price_decay_over_versions);
            },
            */
        }
    }

    pub fn read(bytes: &mut Cursor<Bytes>) -> Result<Msg, MsgReadError> {
        if bytes.remaining() < 2 {
            return Err(MsgReadError::Truncated);
        }

        let tag = bytes.get_u16_be();
        match tag {
            /*
            tag::SENDER_GET_MUTABLE => {
                if bytes.remaining() < 32 + 3 * 8 {
                    return Err(MsgReadError::Truncated);
                }

                let id = PublicSignKey::from_bytes(slice_to_array!(&bytes.get_ref()[..32], 32));
                bytes.advance(32);
                let price = Btc(bytes.get_f64_be());
                let price_decay_over_time = Sec(bytes.get_f64_be());
                let price_decay_over_versions = bytes.get_f64_be();
                let params = GetMutableParams {
                    price, price_decay_over_time, price_decay_over_versions,
                };
                Ok(Msg::SenderGetMutable { id, params })
            },
            */
            _ => Err(MsgReadError::InvalidMsgKind),
        }
    }
}

#[derive(Debug, Fail)]
pub enum MsgReadError {
    #[fail(display = "message too short")]
    Truncated,
    #[fail(display = "invalid message kind")]
    InvalidMsgKind,
}

impl OutgoingMsg {
    pub fn utility_decay_at(&self, at: Instant) -> Btc {
        let time = Sec::from(at.duration_since(self.utility_time));
        self.utility * (- time / self.utility_decay).exp()
    }
}

