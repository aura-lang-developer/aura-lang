//! Cryptographic primitives and utilities for Aura Runtime.
//!
//! Provides pure-Rust implementations with zero external dependencies:
//! - SHA-256 (`Sha256`, `sha256_hash`, `sha256_hex`)
//! - HMAC-SHA-256 (`hmac_sha256`)
//! - PBKDF2-HMAC-SHA-256 (`pbkdf2_hmac_sha256`)
//! - Base64 Encoding & Decoding (`base64_encode`, `base64_decode`)

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Incremental SHA-256 state machine.
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buffer: [0u8; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.total_len += data.len() as u64;
        let mut offset = 0;
        while offset < data.len() {
            let space = 64 - self.buf_len;
            let take = (data.len() - offset).min(space);
            self.buffer[self.buf_len..self.buf_len + take]
                .copy_from_slice(&data[offset..offset + take]);
            self.buf_len += take;
            offset += take;

            if self.buf_len == 64 {
                self.process_block();
                self.buf_len = 0;
            }
        }
    }

    fn process_block(&mut self) {
        let mut w = [0u32; 64];
        for (i, item) in w.iter_mut().take(16).enumerate() {
            *item = u32::from_be_bytes([
                self.buffer[i * 4],
                self.buffer[i * 4 + 1],
                self.buffer[i * 4 + 2],
                self.buffer[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }

    pub fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.total_len * 8;
        // Pad with 0x80
        self.buffer[self.buf_len] = 0x80;
        self.buf_len += 1;

        if self.buf_len > 56 {
            for i in self.buf_len..64 {
                self.buffer[i] = 0;
            }
            self.process_block();
            self.buf_len = 0;
        }

        for i in self.buf_len..56 {
            self.buffer[i] = 0;
        }

        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        self.process_block();

        let mut out = [0u8; 32];
        for (i, word) in self.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

/// Computes the SHA-256 hash of a byte slice.
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

/// Computes the SHA-256 hash formatted as a lower-case hexadecimal string.
pub fn sha256_hex(data: &[u8]) -> String {
    let bytes = sha256_hash(data);
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Computes HMAC-SHA-256 according to RFC 2104.
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let hashed = sha256_hash(key);
        k[..32].copy_from_slice(&hashed);
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut o_key_pad = [0x5cu8; 64];
    let mut i_key_pad = [0x36u8; 64];
    for i in 0..64 {
        o_key_pad[i] ^= k[i];
        i_key_pad[i] ^= k[i];
    }
    let mut inner = Vec::with_capacity(64 + message.len());
    inner.extend_from_slice(&i_key_pad);
    inner.extend_from_slice(message);
    let inner_hash = sha256_hash(&inner);

    let mut outer = Vec::with_capacity(64 + 32);
    outer.extend_from_slice(&o_key_pad);
    outer.extend_from_slice(&inner_hash);
    sha256_hash(&outer)
}

/// Computes PBKDF2 with HMAC-SHA-256 key derivation (1 block, 32 bytes output).
pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut initial_salt = salt.to_vec();
    initial_salt.extend_from_slice(&1u32.to_be_bytes());
    let mut u = hmac_sha256(password, &initial_salt);
    let mut out = u;

    for _ in 1..iterations {
        u = hmac_sha256(password, &u);
        for i in 0..32 {
            out[i] ^= u[i];
        }
    }
    out
}

/// Encodes binary data to standard Base64 string with padding.
pub fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut res = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        res.push(TABLE[(b0 >> 2) as usize] as char);
        res.push(TABLE[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            res.push(TABLE[(((b1 & 15) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            res.push('=');
        }
        if chunk.len() > 2 {
            res.push(TABLE[(b2 & 63) as usize] as char);
        } else {
            res.push('=');
        }
    }
    res
}

/// Decodes standard Base64 string into bytes.
pub fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.len() % 4 != 0 {
        return None;
    }
    let decode_byte = |b: u8| -> Option<u8> {
        match b {
            b'A'..=b'Z' => Some(b - b'A'),
            b'a'..=b'z' => Some(b - b'a' + 26),
            b'0'..=b'9' => Some(b - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            b'=' => Some(0),
            _ => None,
        }
    };
    let mut out = Vec::with_capacity((clean.len() / 4) * 3);
    for chunk in clean.chunks_exact(4) {
        let b0 = decode_byte(chunk[0])?;
        let b1 = decode_byte(chunk[1])?;
        let b2 = decode_byte(chunk[2])?;
        let b3 = decode_byte(chunk[3])?;

        let triple = ((b0 as u32) << 18) | ((b1 as u32) << 12) | ((b2 as u32) << 6) | (b3 as u32);
        out.push(((triple >> 16) & 0xff) as u8);
        if chunk[2] != b'=' {
            out.push(((triple >> 8) & 0xff) as u8);
        }
        if chunk[3] != b'=' {
            out.push((triple & 0xff) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"hello world"),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hmac_and_scram_vectors() {
        let hmac = hmac_sha256(b"key", b"The quick brown fox jumps over the lazy dog");
        let hmac_hex: String = hmac.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(
            hmac_hex,
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );

        let rfc_salt = base64_decode("W22ZaJ0SNY7soNVgvFoSnQ==").unwrap();
        let salted_pw = pbkdf2_hmac_sha256(b"pencil", &rfc_salt, 4096);
        let client_key = hmac_sha256(&salted_pw, b"Client Key");
        let stored_key = sha256_hash(&client_key);
        let stored_key_hex: String = stored_key.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(
            stored_key_hex,
            "a07156c3493aec5e8b30542f283009d2c210b21ada2ca3f061dfd440162cea2c"
        );
    }

    #[test]
    fn test_base64_roundtrip() {
        let sample = b"hello aura postgres and crypto!";
        let encoded = base64_encode(sample);
        let decoded = base64_decode(&encoded).expect("Decode should succeed");
        assert_eq!(decoded, sample);
    }
}
