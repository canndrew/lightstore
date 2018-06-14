pub const PROTOCOL_VERSION: u32 = 70015;
pub const MAGIC_MAINNET: u32 = 0xd9b4bef9;
pub const USER_AGENT: &'static str = "/ass_balls:0.0.0";

mod peer;
mod msg;

pub use self::peer::*;
pub use self::msg::*;

