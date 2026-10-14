use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MySqlConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: Option<String>,
    pub database: String,
    pub connect_timeout: Duration,
}

impl MySqlConfig {
    pub fn from_conn_str(conn_str: &str) -> Result<Self, String> {
        let mut host = "127.0.0.1".to_string();
        let mut port = 3306u16;
        let mut user = "root".to_string();
        let mut password = None;
        let mut database = "".to_string();

        let s = conn_str.trim();
        if s.starts_with("mysql://") || s.starts_with("mariadb://") {
            let without_scheme = if let Some(stripped) = s.strip_prefix("mysql://") {
                stripped
            } else if let Some(stripped) = s.strip_prefix("mariadb://") {
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
            // Key-value format
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

pub struct MySqlConnection {
    stream: TcpStream,
    seq: u8,
    pub server_version: String,
}

#[derive(Debug, Clone)]
pub struct MySqlRow {
    pub columns: Vec<String>,
    pub values: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct MySqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<MySqlRow>,
    pub rows_affected: i64,
    pub last_insert_id: i64,
}

impl MySqlConnection {
    pub fn connect(config: &MySqlConfig) -> Result<Self, String> {
        let addr_str = format!("{}:{}", config.host, config.port);
        let addrs: Vec<_> = addr_str
            .to_socket_addrs()
            .map_err(|e| format!("Failed to resolve {}: {}", addr_str, e))?
            .collect();

        if addrs.is_empty() {
            return Err(format!("Could not resolve MySQL address {}", addr_str));
        }

        let stream = TcpStream::connect_timeout(&addrs[0], config.connect_timeout)
            .map_err(|e| format!("Failed to connect to MySQL at {}: {}", addr_str, e))?;

        let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        let _ = stream.set_nodelay(true);

        let mut conn = Self {
            stream,
            seq: 0,
            server_version: String::new(),
        };

        conn.handshake(config)?;
        Ok(conn)
    }

    fn read_packet(&mut self) -> Result<Vec<u8>, String> {
        let mut header = [0u8; 4];
        self.stream
            .read_exact(&mut header)
            .map_err(|e| format!("Failed to read MySQL packet header: {}", e))?;

        let len = (header[0] as usize) | ((header[1] as usize) << 8) | ((header[2] as usize) << 16);
        self.seq = header[3].wrapping_add(1);

        let mut body = vec![0u8; len];
        self.stream
            .read_exact(&mut body)
            .map_err(|e| format!("Failed to read MySQL packet body (len {}): {}", len, e))?;

        if !body.is_empty() && body[0] == 0xFF {
            // ERR packet
            let err_code = if body.len() >= 3 {
                u16::from_le_bytes([body[1], body[2]])
            } else {
                0
            };
            let msg = if body.len() > 9 {
                String::from_utf8_lossy(&body[9..]).to_string()
            } else {
                "Unknown MySQL error".to_string()
            };
            return Err(format!("MySQL Error {}: {}", err_code, msg));
        }

        Ok(body)
    }

    fn write_packet(&mut self, payload: &[u8]) -> Result<(), String> {
        let len = payload.len();
        let header = [
            (len & 0xFF) as u8,
            ((len >> 8) & 0xFF) as u8,
            ((len >> 16) & 0xFF) as u8,
            self.seq,
        ];
        self.seq = self.seq.wrapping_add(1);

        self.stream
            .write_all(&header)
            .map_err(|e| format!("Failed to write MySQL header: {}", e))?;
        self.stream
            .write_all(payload)
            .map_err(|e| format!("Failed to write MySQL payload: {}", e))?;
        Ok(())
    }

    fn handshake(&mut self, config: &MySqlConfig) -> Result<(), String> {
        let init_packet = self.read_packet()?;
        if init_packet.is_empty() {
            return Err("Empty handshake packet from MySQL".to_string());
        }

        let mut offset = 1; // Protocol version (10)
        let ver_end = init_packet[offset..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(0);
        self.server_version =
            String::from_utf8_lossy(&init_packet[offset..offset + ver_end]).to_string();
        offset += ver_end + 1 + 4; // Skip version, null, thread_id (4)

        // Auth plugin data part 1 (8 bytes)
        let auth_data_1 = if offset + 8 <= init_packet.len() {
            &init_packet[offset..offset + 8]
        } else {
            &[]
        };
        offset += 8 + 1 + 2 + 1 + 2 + 2 + 1 + 10; // various capability flags and status

        // Auth plugin data part 2 (usually 12 bytes)
        let auth_data_2 = if offset < init_packet.len() {
            let end = init_packet[offset..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(init_packet.len() - offset);
            &init_packet[offset..offset + end]
        } else {
            &[]
        };

        let mut scramble = Vec::new();
        scramble.extend_from_slice(auth_data_1);
        scramble.extend_from_slice(auth_data_2);

        // HandshakeResponse41
        let client_flags: u32 = 0x00000001  // CLIENT_LONG_PASSWORD
            | 0x00000004                    // CLIENT_LONG_FLAG
            | 0x00000200                    // CLIENT_PROTOCOL_41
            | 0x00008000                    // CLIENT_SECURE_CONNECTION
            | if !config.database.is_empty() { 0x00000008 } else { 0 }; // CLIENT_CONNECT_WITH_DB

        let max_packet_size: u32 = 16 * 1024 * 1024;
        let charset: u8 = 33; // utf8_general_ci / 45 for utf8mb4

        let mut resp = Vec::new();
        resp.extend_from_slice(&client_flags.to_le_bytes());
        resp.extend_from_slice(&max_packet_size.to_le_bytes());
        resp.push(charset);
        resp.extend_from_slice(&[0u8; 23]); // Filler

        // Username (null-terminated)
        resp.extend_from_slice(config.user.as_bytes());
        resp.push(0);

        // Auth response (length-encoded or scrambled)
        if let Some(ref pw) = config.password {
            if !pw.is_empty() && !scramble.is_empty() {
                let hashed = scramble_password(pw.as_bytes(), &scramble);
                resp.push(hashed.len() as u8);
                resp.extend_from_slice(&hashed);
            } else {
                resp.push(0);
            }
        } else {
            resp.push(0);
        }

        // Database
        if !config.database.is_empty() {
            resp.extend_from_slice(config.database.as_bytes());
            resp.push(0);
        }

        self.write_packet(&resp)?;

        // Read OK packet or AuthMoreData / AuthSwitch
        let auth_result = self.read_packet()?;
        if !auth_result.is_empty() && auth_result[0] == 0xFE {
            // AuthSwitchRequest - respond with empty/scramble
            let pw = config.password.as_deref().unwrap_or("");
            let hashed = scramble_password(pw.as_bytes(), &scramble);
            self.write_packet(&hashed)?;
            let _ = self.read_packet()?;
        }

        Ok(())
    }

    pub fn query(&mut self, sql: &str) -> Result<MySqlQueryResult, String> {
        self.seq = 0;
        let mut payload = Vec::with_capacity(1 + sql.len());
        payload.push(0x03); // COM_QUERY
        payload.extend_from_slice(sql.as_bytes());

        self.write_packet(&payload)?;

        let first = self.read_packet()?;
        if first.is_empty() {
            return Err("Empty response from MySQL".to_string());
        }

        if first[0] == 0x00 {
            // OK Packet: [0x00, affected_rows (lenenc), last_insert_id (lenenc), status (2), warnings (2)]
            let mut offset = 1;
            let (affected, off) = read_lenenc_int(&first[offset..]);
            offset += off;
            let (insert_id, _) = read_lenenc_int(&first[offset..]);

            return Ok(MySqlQueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                rows_affected: affected as i64,
                last_insert_id: insert_id as i64,
            });
        }

        // Resultset: first byte is column count
        let (num_cols, _) = read_lenenc_int(&first);
        let mut columns = Vec::with_capacity(num_cols as usize);

        for _ in 0..num_cols {
            let col_def = self.read_packet()?;
            // Column definition packet contains [catalog, schema, table, orig_table, name, orig_name, ...]
            let mut offset = 0;
            let (_, off) = read_lenenc_str(&col_def[offset..]); // catalog
            offset += off;
            let (_, off) = read_lenenc_str(&col_def[offset..]); // schema
            offset += off;
            let (_, off) = read_lenenc_str(&col_def[offset..]); // table
            offset += off;
            let (_, off) = read_lenenc_str(&col_def[offset..]); // orig_table
            offset += off;
            let (col_name, _) = read_lenenc_str(&col_def[offset..]); // name
            columns.push(col_name);
        }

        // Skip EOF packet if protocol < 4.1 or Deprecate EOF not set
        let maybe_eof = self.read_packet()?;
        if maybe_eof[0] != 0xFE {
            // If it wasn't EOF, it's a row!
            // Handle row
        }

        let mut rows = Vec::new();
        loop {
            let row_packet = self.read_packet()?;
            if row_packet.is_empty() || row_packet[0] == 0xFE {
                // EOF packet
                break;
            }

            let mut offset = 0;
            let mut row_vals = Vec::with_capacity(columns.len());
            for _ in 0..columns.len() {
                if offset >= row_packet.len() {
                    row_vals.push(None);
                    continue;
                }
                if row_packet[offset] == 0xFB {
                    // NULL
                    row_vals.push(None);
                    offset += 1;
                } else {
                    let (val_str, off) = read_lenenc_str(&row_packet[offset..]);
                    row_vals.push(Some(val_str));
                    offset += off;
                }
            }

            rows.push(MySqlRow {
                columns: columns.clone(),
                values: row_vals,
            });
        }

        Ok(MySqlQueryResult {
            columns,
            rows,
            rows_affected: 0,
            last_insert_id: 0,
        })
    }
}

fn read_lenenc_int(buf: &[u8]) -> (u64, usize) {
    if buf.is_empty() {
        return (0, 0);
    }
    match buf[0] {
        0..=250 => (buf[0] as u64, 1),
        0xFC => {
            if buf.len() >= 3 {
                (u16::from_le_bytes([buf[1], buf[2]]) as u64, 3)
            } else {
                (0, buf.len())
            }
        }
        0xFD => {
            if buf.len() >= 4 {
                let v = (buf[1] as u64) | ((buf[2] as u64) << 8) | ((buf[3] as u64) << 16);
                (v, 4)
            } else {
                (0, buf.len())
            }
        }
        0xFE => {
            if buf.len() >= 9 {
                let mut b = [0u8; 8];
                b.copy_from_slice(&buf[1..9]);
                (u64::from_le_bytes(b), 9)
            } else {
                (0, buf.len())
            }
        }
        _ => (0, 1),
    }
}

fn read_lenenc_str(buf: &[u8]) -> (String, usize) {
    let (len, off) = read_lenenc_int(buf);
    let str_len = len as usize;
    if off + str_len <= buf.len() {
        let s = String::from_utf8_lossy(&buf[off..off + str_len]).to_string();
        (s, off + str_len)
    } else {
        (String::new(), off)
    }
}

// SHA1 Native Scramble for MySQL Native Password Auth
fn scramble_password(password: &[u8], scramble: &[u8]) -> Vec<u8> {
    if password.is_empty() {
        return Vec::new();
    }
    let stage1 = sha1_hash(password);
    let stage2 = sha1_hash(&stage1);

    let mut combined = Vec::new();
    combined.extend_from_slice(&scramble[..20.min(scramble.len())]);
    combined.extend_from_slice(&stage2);
    let stage3 = sha1_hash(&combined);

    let mut result = vec![0u8; 20];
    for i in 0..20 {
        result[i] = stage1[i] ^ stage3[i];
    }
    result
}

// Minimal standard SHA-1 implementation
fn sha1_hash(input: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let bit_len = (input.len() as u64) * 8;
    let mut msg = input.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (i, item) in w.iter().enumerate() {
            let (f, k) = if i < 20 {
                ((b & c) | ((!b) & d), 0x5A827999)
            } else if i < 40 {
                (b ^ c ^ d, 0x6ED9EBA1)
            } else if i < 60 {
                ((b & c) | (b & d) | (c & d), 0x8F1BBCDC)
            } else {
                (b ^ c ^ d, 0xCA62C1D6)
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*item);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut result = [0u8; 20];
    result[0..4].copy_from_slice(&h0.to_be_bytes());
    result[4..8].copy_from_slice(&h1.to_be_bytes());
    result[8..12].copy_from_slice(&h2.to_be_bytes());
    result[12..16].copy_from_slice(&h3.to_be_bytes());
    result[16..20].copy_from_slice(&h4.to_be_bytes());
    result
}
