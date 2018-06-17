use priv_prelude::*;
use rand;
use tokio_io;

use bitcoin::{MAGIC_MAINNET, PROTOCOL_VERSION, USER_AGENT};

pub struct Msg {
    cursor: Cursor<Vec<u8>>,
}

impl Msg {
    pub fn new(command: &str) -> Msg {
        let bytes = Vec::with_capacity(24);
        let mut cursor = Cursor::new(bytes);
        unwrap!(cursor.write_u32::<LittleEndian>(MAGIC_MAINNET));
        let command = {
            let mut command_bytes = [0u8; 12];
            let command_len = command.len();
            command_bytes[..command_len].clone_from_slice(command.as_bytes());
            command_bytes
        };
        unwrap!(cursor.write(&command[..]));
        unwrap!(cursor.write(&[0u8; 8][..]));
        Msg { cursor }
    }

    pub fn finish(mut self) -> Vec<u8> {
        let len = self.cursor.position() as u32 - 24;
        self.cursor.set_position(16);
        unwrap!(self.cursor.write_u32::<LittleEndian>(len));
        let sha = Sha256::digest(&self.cursor.get_ref()[24..]);
        let sha = Sha256::digest(&sha[..]);
        unwrap!(self.cursor.write(&sha[..4]));
        self.cursor.into_inner()
    }

    pub fn remaining(&self) -> usize {
        self.cursor.get_ref().len() - (self.cursor.position() as usize)
    }

    pub fn write_addr(&mut self, service_flags: u64, addr: &SocketAddr) {
        unwrap!(self.write_u64::<LittleEndian>(service_flags));
        let ip = match addr.ip() {
            IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
            IpAddr::V6(ipv6) => ipv6,
        };
        unwrap!(self.write(&ip.octets()));
        unwrap!(self.write_u16::<BigEndian>(addr.port()));
    }

    pub fn write_var_str(&mut self, s: &str) {
        self.write_var_int(s.len() as u64);
        unwrap!(self.write(s.as_bytes()));
    }

    pub fn parse_var_str(&mut self) -> Result<String, ParseVarStrError> {
        let len = {
            self
            .parse_var_int()
            .map_err(ParseVarStrError::ParseLength)?
        } as usize;
        let mut message = Vec::zeros(len);
        if self.remaining() < len {
            return Err(ParseVarStrError::MessageTooShort);
        }
        unwrap!(self.read_exact(&mut message));
        String::from_utf8(message)
        .map_err(ParseVarStrError::MalformedString)
    }

    pub fn write_var_int(&mut self, x: u64) {
        if x < 0xfd {
            unwrap!(self.write(&[x as u8]));
        } else if x < 0xffff {
            unwrap!(self.write(&[0xfd]));
            unwrap!(self.write_u16::<LittleEndian>(x as u16));
        } else if x < 0xffff_ffff {
            unwrap!(self.write(&[0xfe]));
            unwrap!(self.write_u32::<LittleEndian>(x as u32));
        } else {
            unwrap!(self.write(&[0xff]));
            unwrap!(self.write_u64::<LittleEndian>(x));
        }
    }

    pub fn parse_var_int(&mut self) -> Result<u64, ParseVarIntError> {
        if self.remaining() < 1 {
            return Err(ParseVarIntError::MessageTooShort);
        }
        let c = unwrap!(self.read_u8());
        if c < 0xfd {
            return Ok(c as u64);
        }
        if c == 0xfd {
            if self.remaining() < 2 {
                return Err(ParseVarIntError::MessageTooShort);
            }
            return Ok(unwrap!(self.read_u16::<LittleEndian>()) as u64);
        }
        if c == 0xfe {
            if self.remaining() < 4 {
                return Err(ParseVarIntError::MessageTooShort);
            }
            return Ok(unwrap!(self.read_u32::<LittleEndian>()) as u64);
        }
        if self.remaining() < 8 {
            return Err(ParseVarIntError::MessageTooShort);
        }
        Ok(unwrap!(self.read_u64::<LittleEndian>()))
    }

    pub fn version(addr_from: &SocketAddr, addr_to: &SocketAddr) -> Vec<u8> {
        let service_flags = 0;
        let mut msg = Msg::new("version");
        unwrap!(msg.write_u32::<LittleEndian>(PROTOCOL_VERSION));
        unwrap!(msg.write_u64::<LittleEndian>(service_flags));
        let time = unwrap!(SystemTime::now().duration_since(UNIX_EPOCH)).as_secs();
        unwrap!(msg.write_u64::<LittleEndian>(time));
        msg.write_addr(service_flags, addr_from);
        msg.write_addr(service_flags, addr_to);
        let nonce = rand::random();
        unwrap!(msg.write_u64::<LittleEndian>(nonce));
        msg.write_var_str(USER_AGENT);
        let start_height = 1;
        unwrap!(msg.write_u32::<LittleEndian>(start_height));
        unwrap!(msg.write(&[0]));
        msg.finish()
    }

    pub fn read_msg<R: AsyncRead + 'static>(r: R)
        -> impl Future<Item = (R, String, Msg), Error = ReadMsgError>
    {
        let header = [0u8; 24];
        tokio_io::io::read_exact(r, header)
        .map_err(ReadMsgError::Read)
        .and_then(|(r, header)| {
            let mut cursor = Cursor::new(header);
            let magic = unwrap!(cursor.read_u32::<LittleEndian>());
            if magic != MAGIC_MAINNET {
                return future::err(ReadMsgError::InvalidMagicNumber).into_boxed();
            }
            let header = cursor.into_inner();
            let command = {
                let mut command_end = 15;
                while header[command_end] == 0 {
                    command_end -= 1;
                }
                let command = &header[4..(command_end + 1)];
                try_fut!(
                    String::from_utf8(command.to_vec())
                    .map_err(ReadMsgError::InvalidCommand)
                )
            };
            let mut cursor = Cursor::new(&header[16..]);
            let payload_len = unwrap!(cursor.read_u32::<LittleEndian>());
            let mut checksum = [0u8; 4];
            unwrap!(cursor.read_exact(&mut checksum));

            let payload = Vec::zeros(payload_len as usize);
            tokio_io::io::read_exact(r, payload)
            .map_err(ReadMsgError::Read)
            .map(|(r, payload)| {
                let msg = Msg { cursor: Cursor::new(payload) };
                (r, command, msg)
            })
            .into_boxed()
        })
    }

    pub fn parse_ver_ack(&self) -> Result<(), ParseVerAckError> {
        if self.cursor.get_ref().len() != 0 {
            return Err(ParseVerAckError::UnexpectedPayload(self.cursor.get_ref().len()));
        }
        Ok(())
    }

    pub fn parse_reject(&mut self) -> Result<Rejection, ParseRejectError> {
        let message = {
            self
            .parse_var_str()
            .map_err(ParseRejectError::ParseMessage)?
        };
        if self.remaining() < 1 {
            return Err(ParseRejectError::MessageTooShort);
        }
        let code = unwrap!(self.read_u8());
        let code = match code {
            0x01 => RejectionCode::Malformed,
            0x10 => RejectionCode::Invalid,
            0x11 => RejectionCode::Obsolete,
            0x12 => RejectionCode::Duplicate,
            0x40 => RejectionCode::Nonstandard,
            0x41 => RejectionCode::Dust,
            0x42 => RejectionCode::InsufficientFee,
            0x43 => RejectionCode::Checkpoint,
            _ => return Err(ParseRejectError::InvalidRejectionCode(code)),
        };
        let reason = {
            self
            .parse_var_str()
            .map_err(ParseRejectError::ParseReason)?
        };
        let mut data = Vec::zeros(self.remaining());
        unwrap!(self.read_exact(&mut data));
        Ok(Rejection {
            message, code, reason, data,
        })
    }

    pub fn parse_version(&mut self) -> Result<(), ParseVersionError> {
        if self.remaining() < 80 {
            return Err(ParseVersionError::MessageTooShort);
        }
        let _version = unwrap!(self.read_u32::<LittleEndian>());
        let _services = unwrap!(self.read_u64::<LittleEndian>());
        let _timestamp = unwrap!(self.read_u64::<LittleEndian>());

        let _our_services = unwrap!(self.read_u64::<LittleEndian>());
        let mut our_ip = [0u8; 16];
        unwrap!(self.read_exact(&mut our_ip));
        let _our_ip = Ipv6Addr::from(our_ip);
        let _our_port = unwrap!(self.read_u16::<BigEndian>());

        let _their_services = unwrap!(self.read_u64::<LittleEndian>());
        let mut their_ip = [0u8; 16];
        unwrap!(self.read_exact(&mut their_ip));
        let _their_ip = Ipv6Addr::from(their_ip);
        let _their_port = unwrap!(self.read_u16::<BigEndian>());

        let _nonce = unwrap!(self.read_u64::<BigEndian>());

        let _user_agent = self.parse_var_str().map_err(ParseVersionError::ParseUserAgent)?;

        if self.remaining() < 4 {
            return Err(ParseVersionError::MessageTooShort);
        }
        let _start_height = unwrap!(self.read_u32::<LittleEndian>());

        if self.remaining() > 0 {
            let _relay = unwrap!(self.read_u8());
            if self.remaining() > 0 {
                return Err(ParseVersionError::MessageTooLong);
            }
        }

        Ok(())
    }
}

impl Write for Msg {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.cursor.write(data)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for Msg {
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(data)
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum ReadMsgError {
        Read(e: io::Error) {
            description("error reading message from remote peer")
            display("error reading message from remote peer: {}", e)
            cause(e)
        }
        InvalidMagicNumber {
            description("invalid magic number at start of message")
        }
        InvalidCommand(e: FromUtf8Error) {
            description("malformed command string")
            display("malformed command string: {}", e)
            cause(e)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseVerAckError {
        UnexpectedPayload(len: usize) {
            description("unexpected payload")
            display("unexpected payload. Expected empty payload, got payload of length {}", len)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseRejectError {
        ParseMessage(e: ParseVarStrError) {
            description("error parsing message")
            display("error parsing message: {}", e)
            cause(e)
        }
        MessageTooShort {
            description("message too short")
        }
        ParseReason(e: ParseVarStrError) {
            description("error parsing reason")
            display("error parsing reason: {}", e)
            cause(e)
        }
        InvalidRejectionCode(code: u8) {
            description("invalid rejection code")
            display("invalid rejection code: {}", code)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseVarStrError {
        ParseLength(e: ParseVarIntError) {
            description("error parsing length")
            display("error parsing length: {}", e)
            cause(e)
        }
        MessageTooShort {
            description("message too short")
        }
        MalformedString(e: FromUtf8Error) {
            description("malformed string")
            display("maformed string: {}", e)
            cause(e)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseVarIntError {
        MessageTooShort {
            description("message too short")
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseVersionError {
        MessageTooShort {
            description("message too short")
        }
        MessageTooLong {
            description("message too long")
        }
        ParseUserAgent(e: ParseVarStrError) {
            description("error parsing user-agent field")
            display("error parsing user-agent field: {}", e)
            cause(e)
        }
    }
}

#[derive(Debug)]
pub struct Rejection {
    message: String,
    reason: String,
    code: RejectionCode,
    data: Vec<u8>
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rejected {}. {} (code {})", self.message, self.reason, match self.code {
            RejectionCode::Malformed => "malformed",
            RejectionCode::Invalid => "invalid",
            RejectionCode::Obsolete => "obsolete",
            RejectionCode::Duplicate => "duplicate",
            RejectionCode::Nonstandard => "nonstandard",
            RejectionCode::Dust => "dust",
            RejectionCode::InsufficientFee => "insufficient fee",
            RejectionCode::Checkpoint => "checkpoint",
        })
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum RejectionCode {
    Malformed = 0x01,
    Invalid = 0x10,
    Obsolete = 0x11,
    Duplicate = 0x12,
    Nonstandard = 0x40,
    Dust = 0x41,
    InsufficientFee = 0x42,
    Checkpoint = 0x43,
}

