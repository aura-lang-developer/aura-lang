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
                                let col_name =
                                    String::from_utf8_lossy(&body[offset..offset + pos])
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
                                    let val_str =
                                        String::from_utf8_lossy(val_bytes).to_string();
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
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20,
        5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
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
