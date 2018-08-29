use super::*;

pub struct OutgoingPacket {
    pub data: Bytes,
    pub dest: SocketAddr,
    pub utility: Btc,
    pub utility_decay: Sec,
    pub utility_time: Instant,
}

impl OutgoingPacket {
    pub fn new(
        data: Bytes,
        dest: SocketAddr,
        utility: Btc,
        utility_decay: Sec,
    ) -> OutgoingPacket {
        OutgoingPacket {
            data,
            dest,
            utility,
            utility_decay,
            utility_time: Instant::now(),
        }
    }

    pub fn utility_decay_at(&self, at: Instant) -> BtcPerSec {
        let time = Sec::from(at.duration_since(self.utility_time));
        self.utility * (-1.0 / self.utility_decay) * (- time / self.utility_decay).exp()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn dest(&self) -> &SocketAddr {
        &self.dest
    }
}

