pub const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const BASE: u64 = 62;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    #[error("short code was empty")]
    Empty,
    #[error("short code contained an invalid character: {0:?}")]
    InvalidChar(char),
}

pub fn encode(mut id: u64) -> String {
    if id == 0 {
        return (ALPHABET[0] as char).to_string();
    }
    let mut result: String = String::new();
    while id > 0 {
        result.push(ALPHABET[(id % BASE) as usize] as char);
        id /= 62;
    }
    result.chars().rev().collect()
}

pub fn decode(code: &str) -> Result<u64, DecodeError> {
    if code.is_empty() {
        return Err(DecodeError::Empty);
    }
    let mut acc: u64 = 0u64;
    for &byte in code.as_bytes() {
        let digit = match ALPHABET.iter().position(|&c| c == byte) {
            Some(i) => i as u64,
            None => return Err(DecodeError::InvalidChar(byte as char)),
        };
        acc = acc * BASE + digit;
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_small_ids() {
        assert_eq!(encode(0), "0");
        assert_eq!(encode(1), "1");
        assert_eq!(encode(61), "Z");
        assert_eq!(encode(62), "10");
    }

    #[test]
    fn decodes_back_to_id() {
        assert_eq!(decode("0"), Ok(0));
        assert_eq!(decode("Z"), Ok(61));
        assert_eq!(decode("10"), Ok(62));
    }

    #[test]
    fn round_trips_any_id() {
        for id in [0u64, 1, 42, 1000, 999_999, u64::MAX] {
            assert_eq!(decode(&encode(id)), Ok(id), "round trip failed for {id}");
        }
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(decode(""), Err(DecodeError::Empty));
        assert_eq!(decode("abc!"), Err(DecodeError::InvalidChar('!')));
    }
}
