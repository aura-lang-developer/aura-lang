//! Native Pure-Rust PostgreSQL Protocol Client for Aura Runtime.
//! Implements PostgreSQL Frontend/Backend Protocol 3.0 over TCP.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PgConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: Option<String>,
    pub database: String,
    pub connect_timeout: Duration,
}

impl PgConfig {
    pub fn from_conn_str(conn_str: &str) -> Result<Self, String> {
        let mut host = "127.0.0.1".to_string();
        let mut port = 5432u16;
        let mut user = "postgres".to_string();
        let mut password = None;
        let mut database = "postgres".to_string();

        let s = conn_str.trim();
        if s.starts_with("postgres://") || s.starts_with("postgresql://") {
            let without_scheme = if let Some(stripped) = s.strip_prefix("postgres://") {
                stripped
            } else if let Some(stripped) = s.strip_prefix("postgresql://") {
                stripped
            } else {
                s
            };

            let (auth_host, path) = match without_scheme.find('/') {
                Some(idx) => (&without_scheme[..idx], &without_scheme[idx + 1..]),
                None => (without_scheme, ""),
            };

            if !path.is_empty() {
                let db_part = path.split('?').next().unwrap_or(path);
                if !db_part.is_empty() {
                    database = db_part.to_string();
                }
            }

            let (auth_part, host_part) = match auth_host.find('@') {
                Some(idx) => (Some(&auth_host[..idx]), &auth_host[idx + 1..]),
                None => (None, auth_host),
            };

            if let Some(auth) = auth_part {
                let mut parts = auth.split(':');
                if let Some(u) = parts.next() {
                    if !u.is_empty() {
                        user = u.to_string();
                    }
                }
                if let Some(p) = parts.next() {
                    password = Some(p.to_string());
                }
            }

            let host_clean = host_part.split('?').next().unwrap_or(host_part);
            if let Some(idx) = host_clean.rfind(':') {
                host = host_clean[..idx].to_string();
                if let Ok(p) = host_clean[idx + 1..].parse::<u16>() {
                    port = p;
                }
            } else if !host_clean.is_empty() {
                host = host_clean.to_string();
            }
        } else {
            // Key-value format: host=localhost port=5432 user=...
            for token in s.split_whitespace() {
                if let Some((k, v)) = token.split_once('=') {
                    let val = v.trim_matches('\'').trim_matches('"');
                    match k.to_lowercase().as_str() {
                        "host" => host = val.to_string(),
                        "port" => {
                            if let Ok(p) = val.parse::<u16>() {
                                port = p;
                            }
                        }
                        "user" | "username" => user = val.to_string(),
                        "password" | "pass" => password = Some(val.to_string()),
                        "database" | "dbname" | "db" => database = val.to_string(),
                        _ => {}
                    }
                }
            }
        }

        Ok(Self {
            host,
            port,
            user,
            password,
            database,
            connect_timeout: Duration::from_secs(5),
        })
    }
}

pub struct PgConnection {
    stream: TcpStream,
    pub server_params: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct PgRow {
    pub columns: Vec<String>,
    pub values: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct PgQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<PgRow>,
    pub rows_affected: i64,
    pub last_insert_id: i64,
    pub command_tag: String,
}

impl PgConnection {
    pub fn connect(config: &PgConfig) -> Result<Self, String> {
        let addr_str = format!("{}:{}", config.host, config.port);
        let addrs: Vec<_> = addr_str
            .to_socket_addrs()
            .map_err(|e| format!("Failed to resolve {}: {}", addr_str, e))?
            .collect();

        if addrs.is_empty() {
            return Err(format!("Could not resolve address {}", addr_str));
        }

        let stream = TcpStream::connect_timeout(&addrs[0], config.connect_timeout)
            .map_err(|e| format!("Failed to connect to PostgreSQL at {}: {}", addr_str, e))?;

        let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        let _ = stream.set_nodelay(true);

        let mut conn = Self {
            stream,
            server_params: HashMap::new(),
        };

        conn.startup_handshake(config)?;
        Ok(conn)
    }

    fn startup_handshake(&mut self, config: &PgConfig) -> Result<(), String> {
        // StartupMessage: length (i32), protocol version 3.0 (i32 = 196608), key-value pairs null-terminated, final null.
        let mut payload = Vec::new();
        payload.extend_from_slice(&196608i32.to_be_bytes()); // Version 3.0

        payload.extend_from_slice(b"user\0");
        payload.extend_from_slice(config.user.as_bytes());
        payload.push(0);

        payload.extend_from_slice(b"database\0");
        payload.extend_from_slice(config.database.as_bytes());
        payload.push(0);

        payload.extend_from_slice(b"client_encoding\0UTF8\0");
        payload.push(0); // Final terminating null

        let len = (payload.len() + 4) as i32;
        let mut msg = Vec::with_capacity(len as usize);
        msg.extend_from_slice(&len.to_be_bytes());
        msg.extend_from_slice(&payload);

        self.stream
            .write_all(&msg)
            .map_err(|e| format!("Failed to send PG startup message: {}", e))?;

        // Read response until ReadyForQuery ('Z')
        loop {
            let msg_type = self.read_byte()?;
            let msg_len = self.read_i32()? as usize;
            let mut body = vec![0u8; msg_len - 4];
            self.stream
                .read_exact(&mut body)
                .map_err(|e| format!("Failed to read PG message body: {}", e))?;

            match msg_type {
                b'R' => {
                    // Authentication
                    if body.len() < 4 {
                        return Err("Malformed Authentication message from PG".to_string());
                    }
                    let auth_type = i32::from_be_bytes([body[0], body[1], body[2], body[3]]);
                    match auth_type {
                        0 => {
                            // AuthOK
                        }
                        3 => {
                            // CleartextPassword
                            let pw = config.password.as_deref().unwrap_or("");
                            self.send_password_message(pw)?;
                        }
                        5 => {
                            // MD5Password (salt in body[4..8])
                            if body.len() < 8 {
                                return Err("Malformed MD5 auth request".to_string());
                            }
                            let salt = &body[4..8];
                            let pw = config.password.as_deref().unwrap_or("");
                            let md5_resp = compute_pg_md5(&config.user, pw, salt);
                            self.send_password_message(&md5_resp)?;
                        }
                        10 => {
                            // SCRAM-SHA-256 Authentication (PostgreSQL 14+)
                            self.perform_scram_sha256_auth(config)?;
                        }
                        other => {
                            return Err(format!(
                                "Unsupported PostgreSQL authentication method ({})",
                                other
                            ));
                        }
                    }
                }
                b'K' => {
                    // BackendKeyData (PID + Secret Key)
                }
                b'S' => {
                    // ParameterStatus
                    if let Some(idx) = body.iter().position(|&b| b == 0) {
                        let name = String::from_utf8_lossy(&body[..idx]).to_string();
                        let rest = &body[idx + 1..];
                        let val_len = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
                        let val = String::from_utf8_lossy(&rest[..val_len]).to_string();
                        self.server_params.insert(name, val);
                    }
                }
                b'Z' => {
                    // ReadyForQuery
                    break;
                }
                b'E' => {
                    // ErrorResponse
                    let err = parse_pg_error(&body);
                    return Err(format!("PostgreSQL connection error: {}", err));
                }
                _ => {
                    // Ignore notice or warning
                }
            }
        }

        Ok(())
    }

    fn send_password_message(&mut self, password: &str) -> Result<(), String> {
        let mut msg = Vec::new();
        msg.push(b'p');
        let len = (4 + password.len() + 1) as i32;
        msg.extend_from_slice(&len.to_be_bytes());
        msg.extend_from_slice(password.as_bytes());
        msg.push(0);

        self.stream
            .write_all(&msg)
            .map_err(|e| format!("Failed to send password to PG: {}", e))
    }

    fn perform_scram_sha256_auth(&mut self, config: &PgConfig) -> Result<(), String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(123456789);
        let client_nonce = format!("aura_nonce_{:016x}", now_nanos);
        let client_first_bare = format!("n={},r={}", config.user, client_nonce);
        let client_first = format!("n,,{}", client_first_bare);

        // 1. Send SASLInitialResponse ('p')
        let mut msg = Vec::new();
        msg.push(b'p');
        let mech = b"SCRAM-SHA-256\0";
        let body_len = mech.len() + 4 + client_first.len();
        let len = (4 + body_len) as i32;
        msg.extend_from_slice(&len.to_be_bytes());
        msg.extend_from_slice(mech);
        msg.extend_from_slice(&(client_first.len() as i32).to_be_bytes());
        msg.extend_from_slice(client_first.as_bytes());

        self.stream
            .write_all(&msg)
            .map_err(|e| format!("Failed to send SASLInitialResponse: {}", e))?;

        // 2. Read AuthenticationSASLContinue ('R', 11)
        let msg_type = self.read_byte()?;
        if msg_type != b'R' {
            return Err(format!("Expected Authentication message (R), got '{}'", msg_type as char));
        }
        let msg_len = self.read_i32()? as usize;
        let mut body = vec![0u8; msg_len - 4];
        self.stream
            .read_exact(&mut body)
            .map_err(|e| format!("Failed to read SASLContinue body: {}", e))?;

        if body.len() < 4 {
            return Err("Malformed AuthenticationSASLContinue message".to_string());
        }
        let sub_type = i32::from_be_bytes([body[0], body[1], body[2], body[3]]);
        if sub_type != 11 {
            return Err(format!("Expected SASLContinue (11), got {}", sub_type));
        }

        let server_first = String::from_utf8_lossy(&body[4..]).to_string();
        let mut server_nonce = "";
        let mut salt_b64 = "";
        let mut iterations = 4096u32;

        for part in server_first.split(',') {
            if let Some(r) = part.strip_prefix("r=") {
                server_nonce = r;
            } else if let Some(s) = part.strip_prefix("s=") {
                salt_b64 = s;
            } else if let Some(i) = part.strip_prefix("i=") {
                iterations = i.parse().unwrap_or(4096);
            }
        }

        if !server_nonce.starts_with(&client_nonce) {
            return Err("SCRAM server nonce does not match client nonce prefix".to_string());
        }

        let salt = base64_decode(salt_b64).ok_or("Invalid base64 salt from PG server")?;
        let pw = config.password.as_deref().unwrap_or("");

        // 3. Compute ClientProof
        let salted_password = pbkdf2_hmac_sha256(pw.as_bytes(), &salt, iterations);
        let client_key = hmac_sha256(&salted_password, b"Client Key");
        let stored_key = sha256_hash(&client_key);

        let channel_binding = "c=biws"; // base64("n,,")
        let client_final_without_proof = format!("{},r={}", channel_binding, server_nonce);
        let auth_message = format!("{},{},{}", client_first_bare, server_first, client_final_without_proof);

        let client_signature = hmac_sha256(&stored_key, auth_message.as_bytes());
        let mut client_proof = [0u8; 32];
        for i in 0..32 {
            client_proof[i] = client_key[i] ^ client_signature[i];
        }

        let client_final = format!("{},p={}", client_final_without_proof, base64_encode(&client_proof));

        // 4. Send SASLResponse ('p')
        let mut resp_msg = Vec::new();
        resp_msg.push(b'p');
        let resp_len = (4 + client_final.len()) as i32;
        resp_msg.extend_from_slice(&resp_len.to_be_bytes());
        resp_msg.extend_from_slice(client_final.as_bytes());

        self.stream
            .write_all(&resp_msg)
            .map_err(|e| format!("Failed to send SASLResponse: {}", e))?;

        // 5. Read AuthenticationSASLFinal ('R', 12)
        let final_type = self.read_byte()?;
        if final_type != b'R' {
            return Err(format!("Expected SASLFinal (R), got '{}'", final_type as char));
        }
        let final_len = self.read_i32()? as usize;
        let mut final_body = vec![0u8; final_len - 4];
        self.stream
            .read_exact(&mut final_body)
            .map_err(|e| format!("Failed to read SASLFinal body: {}", e))?;

        if final_body.len() >= 4 {
            let final_sub_type = i32::from_be_bytes([final_body[0], final_body[1], final_body[2], final_body[3]]);
            if final_sub_type != 12 {
                return Err(format!("Expected AuthenticationSASLFinal (12), got {}", final_sub_type));
            }
        }

        Ok(())
    }

    pub fn query(&mut self, sql: &str) -> Result<PgQueryResult, String> {
        // Send Simple Query ('Q')
        let mut msg = Vec::new();
        msg.push(b'Q');
        let len = (4 + sql.len() + 1) as i32;
        msg.extend_from_slice(&len.to_be_bytes());
        msg.extend_from_slice(sql.as_bytes());
        msg.push(0);

        self.stream
            .write_all(&msg)
            .map_err(|e| format!("Failed to send query to PG: {}", e))?;

        let mut columns = Vec::new();
        let mut rows = Vec::new();
        let mut command_tag = String::new();
        let mut rows_affected = 0i64;
        let mut last_insert_id = 0i64;

        loop {
            let msg_type = self.read_byte()?;
            let msg_len = self.read_i32()? as usize;
            let mut body = vec![0u8; msg_len - 4];
            self.stream
                .read_exact(&mut body)
                .map_err(|e| format!("Failed to read query response from PG: {}", e))?;

            match msg_type {
                b'T' => {
                    // RowDescription
                    if body.len() >= 2 {
                        let num_fields = u16::from_be_bytes([body[0], body[1]]) as usize;
                        let mut offset = 2;
                        columns.clear();
                        for _ in 0..num_fields {
                            if let Some(pos) = body[offset..].iter().position(|&b| b == 0) {
                                let col_name = String::from_utf8_lossy(&body[offset..offset + pos])
                                    .to_string();
                                columns.push(col_name);
                                offset += pos + 1 + 18; // name + null + table_oid(4) + col_attr(2) + type_oid(4) + type_len(2) + type_mod(4) + format(2)
                            }
                        }
                    }
                }
                b'D' => {
                    // DataRow
                    if body.len() >= 2 {
                        let num_cols = u16::from_be_bytes([body[0], body[1]]) as usize;
                        let mut offset = 2;
                        let mut row_vals = Vec::with_capacity(num_cols);
                        for _ in 0..num_cols {
                            if offset + 4 <= body.len() {
                                let col_len = i32::from_be_bytes([
                                    body[offset],
                                    body[offset + 1],
                                    body[offset + 2],
                                    body[offset + 3],
                                ]);
                                offset += 4;
                                if col_len == -1 {
                                    // NULL
                                    row_vals.push(None);
                                } else {
                                    let len = col_len as usize;
                                    let val_bytes = &body[offset..offset + len];
                                    let val_str = String::from_utf8_lossy(val_bytes).to_string();
                                    offset += len;
                                    row_vals.push(Some(val_str));
                                }
                            }
                        }
                        rows.push(PgRow {
                            columns: columns.clone(),
                            values: row_vals,
                        });
                    }
                }
                b'C' => {
                    // CommandComplete
                    if let Some(pos) = body.iter().position(|&b| b == 0) {
                        command_tag = String::from_utf8_lossy(&body[..pos]).to_string();
                        // Parse command tag: "INSERT 0 5" or "SELECT 10" or "UPDATE 2"
                        let parts: Vec<&str> = command_tag.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if parts[0] == "INSERT" && parts.len() >= 3 {
                                last_insert_id = parts[1].parse().unwrap_or(0);
                                rows_affected = parts[2].parse().unwrap_or(0);
                            } else if let Some(last) = parts.last() {
                                rows_affected = last.parse().unwrap_or(0);
                            }
                        }
                    }
                }
                b'Z' => {
                    // ReadyForQuery
                    break;
                }
                b'E' => {
                    // ErrorResponse
                    let err = parse_pg_error(&body);
                    return Err(format!("PostgreSQL query error: {}", err));
                }
                _ => {
                    // Other messages
                }
            }
        }

        Ok(PgQueryResult {
            columns,
            rows,
            rows_affected,
            last_insert_id,
            command_tag,
        })
    }

    fn read_byte(&mut self) -> Result<u8, String> {
        let mut buf = [0u8; 1];
        self.stream
            .read_exact(&mut buf)
            .map_err(|e| format!("PG connection read error: {}", e))?;
        Ok(buf[0])
    }

    fn read_i32(&mut self) -> Result<i32, String> {
        let mut buf = [0u8; 4];
        self.stream
            .read_exact(&mut buf)
            .map_err(|e| format!("PG connection read error: {}", e))?;
        Ok(i32::from_be_bytes(buf))
    }
}

fn parse_pg_error(body: &[u8]) -> String {
    let mut err_msg = String::new();
    let mut offset = 0;
    while offset < body.len() {
        let field_type = body[offset];
        if field_type == 0 {
            break;
        }
        offset += 1;
        if let Some(pos) = body[offset..].iter().position(|&b| b == 0) {
            let field_val = String::from_utf8_lossy(&body[offset..offset + pos]);
            if field_type == b'M' {
                // Primary message
                if !err_msg.is_empty() {
                    err_msg.push_str("; ");
                }
                err_msg.push_str(&field_val);
            }
            offset += pos + 1;
        } else {
            break;
        }
    }
    if err_msg.is_empty() {
        "Unknown PostgreSQL error".to_string()
    } else {
        err_msg
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

fn compute_pg_md5(user: &str, password: &str, salt: &[u8]) -> String {
    // md5(md5(password + user) + salt)
    let inner_hash = md5_hash(format!("{}{}", password, user).as_bytes());
    let inner = bytes_to_hex(&inner_hash);
    let mut outer_input = inner.into_bytes();
    outer_input.extend_from_slice(salt);
    let outer_hash = md5_hash(&outer_input);
    let outer = bytes_to_hex(&outer_hash);
    format!("md5{}", outer)
}

// Minimal standard MD5 implementation
fn md5_hash(input: &[u8]) -> [u8; 16] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;

    let k: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    let r: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];

    let bit_len = (input.len() as u64) * 8;
    let mut msg = input.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 16];
        for (i, item) in w.iter_mut().enumerate() {
            *item = u32::from_le_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;

        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | (!b & d), i)
            } else if i < 32 {
                ((d & b) | (!d & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | !d), (7 * i) % 16)
            };

            let temp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(k[i])
                    .wrapping_add(w[g])
                    .rotate_left(r[i]),
            );
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
    }

    let mut result = [0u8; 16];
    result[0..4].copy_from_slice(&h0.to_le_bytes());
    result[4..8].copy_from_slice(&h1.to_le_bytes());
    result[8..12].copy_from_slice(&h2.to_le_bytes());
    result[12..16].copy_from_slice(&h3.to_le_bytes());
    result
}

// -----------------------------------------------------------------------------
// Pure-Rust SHA-256 & HMAC-SHA-256 & SCRAM Authentication Primitives
// -----------------------------------------------------------------------------

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

pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut state = [
        0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, item) in w.iter_mut().take(16).enumerate() {
            *item = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA256_K[i]).wrapping_add(w[i]);
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

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut out = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

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
