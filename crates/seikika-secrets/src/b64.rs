//! Small base64 decoder (URL-safe or standard) so remote backends need no extra crate.

use crate::error::SecretError;

pub fn decode(s: &str) -> Result<Vec<u8>, SecretError> {
    let s = s.replace('-', "+").replace('_', "/");
    let padded = match s.len() % 4 {
        2 => format!("{s}=="),
        3 => format!("{s}="),
        _ => s,
    };
    decode_standard(&padded)
}

fn decode_standard(input: &str) -> Result<Vec<u8>, SecretError> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            b'=' => Some(0),
            _ => None,
        }
    }
    let bytes = input.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err(SecretError::backend("b64", "invalid base64 length"));
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let (a, b, c, d) = (
            val(chunk[0]).ok_or_else(|| SecretError::backend("b64", "invalid base64"))?,
            val(chunk[1]).ok_or_else(|| SecretError::backend("b64", "invalid base64"))?,
            val(chunk[2]).ok_or_else(|| SecretError::backend("b64", "invalid base64"))?,
            val(chunk[3]).ok_or_else(|| SecretError::backend("b64", "invalid base64"))?,
        );
        out.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            out.push((b << 4) | (c >> 2));
        }
        if chunk[3] != b'=' {
            out.push((c << 6) | d);
        }
    }
    Ok(out)
}
