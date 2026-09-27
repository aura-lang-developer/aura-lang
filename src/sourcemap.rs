//! Source Maps V3 Generator for the Aura Compiler.
//!
//! Emits standard Source Map V3 JSON (`.js.map`) files and inline comments
//! with Base64 VLQ encoded mappings to support interactive debugging,
//! breakpoints, and stack trace translation directly in `.aura` files.

use std::collections::HashMap;

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes a single integer using Base64 Variable-Length Quantity (VLQ).
pub fn encode_vlq(value: i64) -> String {
    let mut vlq = if value < 0 {
        ((-value) << 1) | 1
    } else {
        value << 1
    };

    let mut result = String::new();
    loop {
        let mut digit = (vlq & 0x1f) as u8;
        vlq >>= 5;
        if vlq > 0 {
            digit |= 0x20; // Set continuation bit
        }
        result.push(BASE64_CHARS[digit as usize] as char);
        if vlq == 0 {
            break;
        }
    }

    result
}

/// A single source mapping entry connecting generated code position to original source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapping {
    pub generated_line: u32,
    pub generated_column: u32,
    pub original_line: u32,
    pub original_column: u32,
}

/// Source Map V3 Generator.
pub struct SourceMapBuilder {
    pub file: String,
    pub source_file: String,
    pub source_content: String,
    pub mappings: Vec<SourceMapping>,
}

impl SourceMapBuilder {
    pub fn new(file: &str, source_file: &str, source_content: &str) -> Self {
        Self {
            file: file.to_string(),
            source_file: source_file.to_string(),
            source_content: source_content.to_string(),
            mappings: Vec::new(),
        }
    }

    /// Adds a mapping from generated (0-indexed) line/col to original (0-indexed) line/col.
    pub fn add_mapping(&mut self, gen_line: u32, gen_col: u32, src_line: u32, src_col: u32) {
        self.mappings.push(SourceMapping {
            generated_line: gen_line,
            generated_column: gen_col,
            original_line: src_line,
            original_column: src_col,
        });
    }

    /// Encodes all mappings into a standard Source Map V3 mappings string.
    pub fn encode_mappings(&self) -> String {
        if self.mappings.is_empty() {
            return String::new();
        }

        // Group mappings by generated line
        let mut line_groups: HashMap<u32, Vec<&SourceMapping>> = HashMap::new();
        let mut max_gen_line = 0;

        for m in &self.mappings {
            line_groups.entry(m.generated_line).or_default().push(m);
            if m.generated_line > max_gen_line {
                max_gen_line = m.generated_line;
            }
        }

        let mut result = String::new();
        let mut prev_src_line = 0i64;
        let mut prev_src_col = 0i64;
        let mut prev_src_file_idx = 0i64;

        for line_idx in 0..=max_gen_line {
            if line_idx > 0 {
                result.push(';');
            }

            if let Some(mut entries) = line_groups.remove(&line_idx) {
                entries.sort_by_key(|e| e.generated_column);

                let mut prev_gen_col = 0i64;
                for (i, entry) in entries.iter().enumerate() {
                    if i > 0 {
                        result.push(',');
                    }

                    // 1. Generated column (relative to previous in this line)
                    let gen_col_diff = (entry.generated_column as i64) - prev_gen_col;
                    prev_gen_col = entry.generated_column as i64;
                    result.push_str(&encode_vlq(gen_col_diff));

                    // 2. Source file index (relative)
                    let src_idx_diff = 0i64 - prev_src_file_idx;
                    prev_src_file_idx = 0;
                    result.push_str(&encode_vlq(src_idx_diff));

                    // 3. Original source line (relative)
                    let src_line_diff = (entry.original_line as i64) - prev_src_line;
                    prev_src_line = entry.original_line as i64;
                    result.push_str(&encode_vlq(src_line_diff));

                    // 4. Original source column (relative)
                    let src_col_diff = (entry.original_column as i64) - prev_src_col;
                    prev_src_col = entry.original_column as i64;
                    result.push_str(&encode_vlq(src_col_diff));
                }
            }
        }

        result
    }

    /// Generates the complete Source Map V3 JSON document.
    pub fn to_json(&self) -> String {
        let mappings_str = self.encode_mappings();
        format!(
            r#"{{"version":3,"file":"{}","sources":["{}"],"sourcesContent":["{}"],"names":[],"mappings":"{}"}}"#,
            escape_json_str(&self.file),
            escape_json_str(&self.source_file),
            escape_json_str(&self.source_content),
            mappings_str
        )
    }

    /// Generates a sourceMappingURL comment for the end of generated JS files.
    pub fn generate_mapping_comment(map_filename: &str) -> String {
        format!("//# sourceMappingURL={}", map_filename)
    }

    /// Generates an inline Data URI sourceMappingURL comment.
    pub fn generate_inline_data_uri(&self) -> String {
        let json = self.to_json();
        let base64_json = base64_encode(json.as_bytes());
        format!(
            "//# sourceMappingURL=data:application/json;charset=utf-8;base64,{}",
            base64_json
        )
    }
}

fn escape_json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i] as u32;
        let b1 = if i + 1 < bytes.len() {
            bytes[i + 1] as u32
        } else {
            0
        };
        let b2 = if i + 2 < bytes.len() {
            bytes[i + 2] as u32
        } else {
            0
        };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(BASE64_CHARS[((triple >> 18) & 0x3f) as usize] as char);
        out.push(BASE64_CHARS[((triple >> 12) & 0x3f) as usize] as char);

        if i + 1 < bytes.len() {
            out.push(BASE64_CHARS[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < bytes.len() {
            out.push(BASE64_CHARS[(triple & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlq_encoding() {
        assert_eq!(encode_vlq(0), "A");
        assert_eq!(encode_vlq(1), "C");
        assert_eq!(encode_vlq(-1), "D");
        assert_eq!(encode_vlq(2), "E");
        assert_eq!(encode_vlq(-2), "F");
        assert_eq!(encode_vlq(16), "gB");
    }

    #[test]
    fn test_source_map_v3_generation() {
        let mut builder = SourceMapBuilder::new("app.js", "app.aura", "fn main() { return 42; }");
        builder.add_mapping(0, 0, 0, 0);
        builder.add_mapping(1, 4, 0, 12);

        let json = builder.to_json();
        assert!(json.contains(r#""version":3"#));
        assert!(json.contains(r#""file":"app.js""#));
        assert!(json.contains(r#""sources":["app.aura"]"#));
        assert!(json.contains(r#""sourcesContent":["fn main() { return 42; }"]"#));
        assert!(json.contains(r#""mappings":""#));

        let comment = SourceMapBuilder::generate_mapping_comment("app.js.map");
        assert_eq!(comment, "//# sourceMappingURL=app.js.map");

        let inline_uri = builder.generate_inline_data_uri();
        assert!(
            inline_uri
                .starts_with("//# sourceMappingURL=data:application/json;charset=utf-8;base64,")
        );
    }
}
