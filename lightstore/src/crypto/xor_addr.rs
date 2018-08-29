use super::*;

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct XorAddr {
    bytes: [u8; 32],
}

impl XorAddr {
    pub const BIT_LEN: u32 = 32;

    pub fn from_bytes(bytes: [u8; 32]) -> XorAddr {
        XorAddr { bytes }
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        self.bytes
    }

    pub fn get_bit(&self, index: u32) -> bool {
        let byte = (index / 8) as usize;
        let offset = 7 - (index % 8);
        let bit = 1 & (self.bytes[byte] >> offset);
        bit == 1
    }

    pub fn set_bit(&mut self, index: u32, val: bool) {
        let byte = (index / 8) as usize;
        let offset = 7 - (index % 8);
        if val {
            self.bytes[byte] |= 1u8 << offset;
        } else {
            self.bytes[byte] &= (!1u8) << offset;
        }
    }

    pub fn clear_bits(&mut self, from: u32) {
        let mut index = from;
        while index % 8 != 0 {
            self.set_bit(index, false);
            index += 1;
        }
        let mut byte = (index / 8) as usize;
        while byte < 32 {
            self.bytes[byte] = 0;
            byte += 1;
        }
    }

    pub fn leading_zeros(&self) -> u32 {
        for b in 0..32u32 {
            if self.bytes[b as usize] != 0 {
                return b * 8 + self.bytes[b as usize].leading_zeros();
            }
        }
        32
    }
}

impl std::ops::BitXor<XorAddr> for XorAddr {
    type Output = XorAddr;

    fn bitxor(self, arg: XorAddr) -> XorAddr {
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = self.bytes[i] ^ arg.bytes[i];
        }
        XorAddr::from_bytes(bytes)
    }
}

impl std::ops::BitXorAssign<XorAddr> for XorAddr {
    fn bitxor_assign(&mut self, arg: XorAddr) {
        for i in 0..32 {
            self.bytes[i] ^= arg.bytes[i];
        }
    }
}

impl fmt::Debug for XorAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = base32::as_base32(&self.as_bytes());
        fmt.debug_tuple("XorAddr").field(&s).finish()
    }
}

