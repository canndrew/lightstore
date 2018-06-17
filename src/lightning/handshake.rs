use priv_prelude::*;
use secp256k1;
use chacha20_poly1305_aead;
use tokio_io;

pub fn send_msg(stream: TcpStream, sk: [u8; 32], msg: Vec<u8>, nonce: u64)
    -> impl Future<Item = TcpStream, Error = io::Error>
{
    let l = {
        let mut l = [0u8; 2];
        {
            let mut cursor = Cursor::new(&mut l[..]);
            unwrap!(cursor.write_u16::<BigEndian>(msg.len() as u16));
        }
        l
    };

    let lc = encrypt_with_ad(&sk, nonce, &[], &l);
    tokio_io::io::write_all(stream, lc)
    .and_then(move |(stream, _lc)| {
        let c = encrypt_with_ad(&sk, nonce, &[], &msg);
        tokio_io::io::write_all(stream, c)
        .map(|(stream, _c)| stream)
    })
}

pub fn recv_msg(stream: TcpStream, rk: [u8; 32], nonce: u64)
    -> impl Future<Item = (TcpStream, Vec<u8>), Error = RecvMsgError>
{
    let lc = [0u8; 18];
    tokio_io::io::read_exact(stream, lc)
    .map_err(RecvMsgError::Io)
    .and_then(move |(stream, lc)| {
        let l = {
            let mut l = try_fut!(
                decrypt_with_ad(&rk, nonce, &[], &lc)
                .ok_or(RecvMsgError::InvalidMsg)
            );

            let mut cursor = Cursor::new(&mut l[..]);
            unwrap!(cursor.read_u16::<BigEndian>())
        };

        let c = Vec::zeros(l as usize + 16);
        tokio_io::io::read_exact(stream, c)
        .map_err(RecvMsgError::Io)
        .and_then(move |(stream, c)| {
            decrypt_with_ad(&rk, nonce, &[], &c)
            .ok_or(RecvMsgError::InvalidMsg)
            .map(move |p| (stream, p))
        })
        .into_boxed()
    })
}

quick_error! {
    #[derive(Debug)]
    pub enum RecvMsgError {
        Io(e: io::Error) {
            description("error reading from socket")
            display("error reading from socket: {}", e)
            cause(e)
        }
        InvalidMsg {
            description("invalid message received from peer")
        }
    }
}

pub fn initiate_handshake(
    secp: Secp256k1,
    stream: TcpStream,
    ls_sk: secp256k1::SecretKey,
    ls_pk: secp256k1::PublicKey,
    rs: secp256k1::PublicKey,
    e_sk: secp256k1::SecretKey,
    e_pk: secp256k1::PublicKey,
) -> impl Future<Item = (TcpStream, [u8; 32], [u8; 32]), Error = HandshakeError>
{
    let (h, ck) = init_handshake_state();
    let h = sha256(&[&h, &rs.serialize()]);

    let h = sha256(&[&h, &e_pk.serialize()]);
    let ss = secp256k1::ecdh::SharedSecret::new(&secp, &rs, &e_sk);
    let (ck, temp_k1)  = hkdf(&ck, &ss[..]);

    let c = encrypt_with_ad(&temp_k1, 0, &h, &[]);

    let h = sha256(&[&h, &c]);
    let mut output = Vec::with_capacity(50);
    output.clear();
    output.push(0);
    output.extend(&e_pk.serialize()[..]);
    output.extend(&c[..]);

    tokio_io::io::write_all(stream, output)
    .map_err(HandshakeError::Io)
    .and_then(move |(stream, _buffer)| {
        let buffer = Vec::zeros(50);
        tokio_io::io::read_exact(stream, buffer)
        .map_err(HandshakeError::Io)
        .and_then(move |(stream, buffer)| {
            let v = buffer[0];
            if v != 0 {
                return future::err(HandshakeError::InvalidResponse).into_boxed();
            }

            let re = slice_to_array!(&buffer[1..34], 33);
            let c = slice_to_array!(&buffer[34..], 16);
            let re = unwrap!(secp256k1::PublicKey::from_slice(&secp, &re[..]));
            let h = sha256(&[&h, &re.serialize()[..]]);
            let ss = secp256k1::ecdh::SharedSecret::new(&secp, &re, &e_sk);
            let (ck, temp_k2) = hkdf(&ck, &ss[..]);
            match decrypt_with_ad(&temp_k2, 0, &h, &c) {
                Some(..) => (),
                None => return future::err(HandshakeError::InvalidResponse).into_boxed(),
            }

            let h = sha256(&[&h, &c]);

            let c = encrypt_with_ad(&temp_k2, 1, &h, &ls_pk.serialize()[..]);
            let h = sha256(&[&h, &c]);
            let ss = secp256k1::ecdh::SharedSecret::new(&secp, &re, &ls_sk);
            let (ck, temp_k3) = hkdf(&ck, &ss[..]);
            let t = encrypt_with_ad(&temp_k3, 0, &h, &[]);
            let (sk, rk) = hkdf(&ck, &[]);

            let mut output = Vec::with_capacity(66);
            output.push(0);
            output.extend(&c);
            output.extend(&t);

            tokio_io::io::write_all(stream, output)
            .map_err(HandshakeError::Io)
            .map(move |(stream, _buffer)| {
                (stream, sk, rk)
            })
            .into_boxed()
        })
    })
}

quick_error! {
    #[derive(Debug)]
    pub enum HandshakeError {
        Io(e: io::Error) {
            description("io error on socket")
            display("io error on socket: {}", e)
            cause(e)
        }
        InvalidResponse {
            description("remote peer sent an invalid response")
        }
        TimedOut {
            description("remote peer took too long to respond")
        }
    }
}

fn init_handshake_state() -> ([u8; 32], [u8; 32]) {
    let h = sha256(&[(b"Noise_XK_secp256k1_ChaChaPoly_SHA256")]);
    let ck = h;
    let h = sha256(&[&h, b"lightning"]);
    let h = slice_to_array!(h, 32);
    let ck = slice_to_array!(ck, 32);
    (h, ck)
}

fn sha256(blocks: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for block in blocks {
        hasher.input(block)
    }
    slice_to_array!(hasher.result(), 32)
}

fn hkdf(salt: &[u8], ikm: &[u8]) -> ([u8; 32], [u8; 32]) {
    let hkdf = Hkdf::<Sha256>::extract(Some(salt), ikm);
    let expanded = hkdf.expand(&[], 64);
    let r0 = slice_to_array!(&expanded[..32], 32);
    let r1 = slice_to_array!(&expanded[32..], 32);
    (r0, r1)
}

fn encode_nonce(n: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    {
        let mut cursor = Cursor::new(&mut nonce[4..]);
        unwrap!(cursor.write_u64::<LittleEndian>(n));
    }
    nonce
}

fn encrypt_with_ad(k: &[u8], n: u64, ad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let nonce = encode_nonce(n);

    let mut v = Vec::new();
    let c = unwrap!(chacha20_poly1305_aead::encrypt(k, &nonce[..], ad, plaintext, &mut v));
    v.extend(&c);
    v
}

fn decrypt_with_ad(k: &[u8], n: u64, ad: &[u8], ciphertext: &[u8]) -> Option<Vec<u8>> {
    let nonce = encode_nonce(n);
    let len = ciphertext.len();

    let mut v = Vec::new();
    match chacha20_poly1305_aead::decrypt(&k, &nonce, ad, &ciphertext[..(len - 16)], &ciphertext[(len - 16)..], &mut v) {
        Ok(()) => Some(v),
        Err(..) => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    //use env_logger;

    #[test]
    fn test_handshake() {
        //let _ = env_logger::init();

        let mut core = unwrap!(Core::new());
        let handle = core.handle();

        let listener = unwrap!(TcpListener::bind(&addr!("0.0.0.0:0"), &handle));
        let listener_addr = unwrap!(listener.local_addr());

        let secp = Secp256k1::new();
        let server_sk = unwrap!(secp256k1::SecretKey::from_slice(&secp, &hex!("2121212121212121212121212121212121212121212121212121212121212121")[..]));
        let server_pk = unwrap!(secp256k1::PublicKey::from_slice(&secp, &hex!("028d7500dd4c12685d1f568b4c2b5048e8534b873319f3a8daa612b469132ec7f7")[..]));
        let client_e_sk = unwrap!(secp256k1::SecretKey::from_slice(&secp, &hex!("1212121212121212121212121212121212121212121212121212121212121212")[..]));
        let client_e_pk = unwrap!(secp256k1::PublicKey::from_slice(&secp, &hex!("036360e856310ce5d294e8be33fc807077dc56ac80d95d9cd4ddbd21325eff73f7")[..]));
        let client_sk = unwrap!(secp256k1::SecretKey::from_slice(&secp, &hex!("1111111111111111111111111111111111111111111111111111111111111111")[..]));
        let client_pk = unwrap!(secp256k1::PublicKey::from_slice(&secp, &hex!("034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa")[..]));

        let client = {
            TcpStream::connect(&listener_addr, &handle)
            .map_err(|e| panic!("error: {}", e))
            .and_then(move |stream| {
                initiate_handshake(secp, stream, client_sk, client_pk, server_pk, client_e_sk, client_e_pk)
                .map_err(|e| panic!("handshake error: {}", e))
                .map(|(_, _, _)| ())
            })
        };

        let server = {
            listener
            .incoming()
            .into_future()
            .map_err(|(e, _incoming)| panic!("oh no: {}", e))
            .and_then(move |(opt, _incoming)| {
                let (stream, _addr) = unwrap!(opt);
                let buffer = Vec::zeros(50);
                tokio_io::io::read_exact(stream, buffer)
                .map_err(|e| panic!("oh no: {}", e))
                .and_then(move |(stream, buffer)| {
                    assert_eq!(buffer, &hex!("00036360e856310ce5d294e8be33fc807077dc56ac80d95d9cd4ddbd21325eff73f70df6086551151f58b8afe6c195782c6a")[..]);

                    let secp = Secp256k1::new();
                    let (h, ck) = init_handshake_state();
                    let h = sha256(&[&h, &server_pk.serialize()[..]]);

                    let v = buffer[0];
                    let re = slice_to_array!(&buffer[1..34], 33);
                    let c = slice_to_array!(&buffer[34..], 16);
                    let re = unwrap!(secp256k1::PublicKey::from_slice(&secp, &re[..]));
                    assert_eq!(v, 0);
                    assert_eq!(&re.serialize()[..], &hex!("036360e856310ce5d294e8be33fc807077dc56ac80d95d9cd4ddbd21325eff73f7")[..]);

                    let h = sha256(&[&h, &re.serialize()[..]]);
                    assert_eq!(h, hex!("9e0e7de8bb75554f21db034633de04be41a2b8a18da7a319a03c803bf02b396c"));

                    let ss = secp256k1::ecdh::SharedSecret::new(&secp, &re, &server_sk);
                    assert_eq!(&ss[..], &hex!("1e2fb3c8fe8fb9f262f649f64d26ecf0f2c0a805a767cf02dc2d77a6ef1fdcc3")[..]);

                    let (ck, temp_k1) = hkdf(&ck, &ss[..]);
                    assert_eq!(ck, hex!("b61ec1191326fa240decc9564369dbb3ae2b34341d1e11ad64ed89f89180582f"));
                    assert_eq!(temp_k1, hex!("e68f69b7f096d7917245f5e5cf8ae1595febe4d4644333c99f9c4a1282031c9f"));

                    unwrap!(decrypt_with_ad(&temp_k1, 0, &h, &c));
                    let h = sha256(&[&h, &c]);
                    assert_eq!(h, hex!("9d1ffbb639e7e20021d9259491dc7b160aab270fb1339ef135053f6f2cebe9ce"));

                    let server_e_sk = unwrap!(secp256k1::SecretKey::from_slice(&secp, &hex!("2222222222222222222222222222222222222222222222222222222222222222")[..]));
                    let server_e_pk = unwrap!(secp256k1::PublicKey::from_slice(&secp, &hex!("02466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f27")[..]));

                    let h = sha256(&[&h, &server_e_pk.serialize()[..]]);
                    assert_eq!(h, hex!("38122f669819f906000621a14071802f93f2ef97df100097bcac3ae76c6dc0bf"));
                    let ss = secp256k1::ecdh::SharedSecret::new(&secp, &re, &server_e_sk);
                    assert_eq!(&ss[..], hex!("c06363d6cc549bcb7913dbb9ac1c33fc1158680c89e972000ecd06b36c472e47"));
                    let (ck, temp_k2) = hkdf(&ck, &ss[..]);
                    assert_eq!(ck, hex!("e89d31033a1b6bf68c07d22e08ea4d7884646c4b60a9528598ccb4ee2c8f56ba"));
                    assert_eq!(temp_k2, hex!("908b166535c01a935cf1e130a5fe895ab4e6f3ef8855d87e9b7581c4ab663ddc"));
			
                    let c = encrypt_with_ad(&temp_k2, 0, &h, &[]);
                    assert_eq!(c, hex!("6e2470b93aac583c9ef6eafca3f730ae"));
                    let h = sha256(&[&h, &c]);
                    assert_eq!(h, hex!("90578e247e98674e661013da3c5c1ca6a8c8f48c90b485c0dfa1494e23d56d72"));

                    let mut output = Vec::with_capacity(50);
                    output.push(0);
                    output.extend(&server_e_pk.serialize()[..]);
                    output.extend(&c);
                    assert_eq!(&output[..], &hex!("0002466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f276e2470b93aac583c9ef6eafca3f730ae")[..]);

                    tokio_io::io::write_all(stream, output)
                    .map_err(|e| panic!(e))
                    .and_then(move |(stream, _buffer)| {
                        let buffer = Vec::zeros(66);
                        tokio_io::io::read_exact(stream, buffer)
                        .map_err(|e| panic!(e))
                        .map(move |(_stream, buffer)| {
                            assert_eq!(buffer[..], hex!("00b9e3a702e93e3a9948c2ed6e5fd7590a6e1c3a0344cfc9d5b57357049aa22355361aa02e55a8fc28fef5bd6d71ad0c38228dc68b1c466263b47fdf31e560e139ba")[..]);

                            /*
                            let v = buffer[0];
                            let c = slice_to_array!(&buffer[1..50], 49);
                            let t = slice_to_array!(&buffer[50..], 16);
                            assert_eq!(v, 0);

                            let rs = decrypt_with_ad(&temp_k2, 1, &h, &c);
                            assert_eq!(rs[..], hex!("034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa")[..]);
                            let rs = unwrap!(secp256k1::PublicKey::from_slice(&secp, &hex!("02466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f27")[..]));
                            let h = sha256(&[&h, &c]);
                            assert_eq!(h, hex!("5dcb5ea9b4ccc755e0e3456af3990641276e1d5dc9afd82f974d90a47c918660"));
                            let ss = secp256k1::ecdh::SharedSecret::new(&secp, &rs, &server_e_sk);
                            assert_eq!(&ss[..], hex!("b36b6d195982c5be874d6d542dc268234379e1ae4ff1709402135b7de5cf0766"));

                            let (ck, temp_k3) = hkdf(&ck, &[]);
                            assert_eq!(ck, hex!("919219dbb2920afa8db80f9a51787a840bcf111ed8d588caf9ab4be716e42b01"));
                            assert_eq!(temp_k3, hex!("981a46c820fb7a241bc8184ba4bb1f01bcdfafb00dde80098cb8c38db9141520"));

                            let p = decrypt_with_ad(&temp_k3, 0, &h, &t);
                            let (rk, sk) = hkdf(&ck, &[]);
                            assert_eq!(rk, hex!("969ab31b4d288cedf6218839b27a3e2140827047f2c0f01bf5c04435d43511a9"));
                            assert_eq!(sk, hex!("bb9020b8965f4df047e07f955f3c4b88418984aadc5cdb35096b9ea8fa5c3442"));
                            */
                        })
                    })
                })
            })
        };

        core.run({
            client.join(server)
            .map(|((), ())| ())
        }).void_unwrap()
    }
}

