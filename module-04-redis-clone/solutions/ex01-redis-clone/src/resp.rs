use crate::error::RespError;
use bytes::Buf;
use std::io::Cursor;

/// RESP (REdis Serialization Protocol) value types
#[derive(Debug, Clone, PartialEq)]
pub enum RespValue {
    /// Simple strings: +OK\r\n
    SimpleString(String),
    /// Errors: -Error message\r\n
    Error(String),
    /// Integers: :1000\r\n
    Integer(i64),
    /// Bulk strings: $6\r\nfoobar\r\n (or $-1\r\n for null)
    BulkString(Option<Vec<u8>>),
    /// Arrays: *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n (or *-1\r\n for null)
    Array(Option<Vec<RespValue>>),
}

impl RespValue {
    /// Parse a RESP value from bytes
    /// Returns the parsed value and the number of bytes consumed
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<RespValue, RespError> {
        if !src.has_remaining() {
            return Err(RespError::Incomplete);
        }

        match src.chunk()[0] {
            b'+' => {
                src.advance(1);
                let line = read_line(src)?;
                Ok(RespValue::SimpleString(line))
            }
            b'-' => {
                src.advance(1);
                let line = read_line(src)?;
                Ok(RespValue::Error(line))
            }
            b':' => {
                src.advance(1);
                let line = read_line(src)?;
                let num = line.parse::<i64>()?;
                Ok(RespValue::Integer(num))
            }
            b'$' => {
                src.advance(1);
                let len_str = read_line(src)?;
                let len = len_str.parse::<i64>()?;

                if len == -1 {
                    return Ok(RespValue::BulkString(None));
                }

                let len = len as usize;
                if src.remaining() < len + 2 {
                    return Err(RespError::Incomplete);
                }

                let data = src.chunk()[..len].to_vec();
                src.advance(len);

                // Consume \r\n
                if src.chunk()[0] != b'\r' || src.chunk()[1] != b'\n' {
                    return Err(RespError::InvalidFormat(
                        "Expected \\r\\n after bulk string".into(),
                    ));
                }
                src.advance(2);

                Ok(RespValue::BulkString(Some(data)))
            }
            b'*' => {
                src.advance(1);
                let len_str = read_line(src)?;
                let len = len_str.parse::<i64>()?;

                if len == -1 {
                    return Ok(RespValue::Array(None));
                }

                let len = len as usize;
                let mut array = Vec::with_capacity(len);

                for _ in 0..len {
                    array.push(RespValue::parse(src)?);
                }

                Ok(RespValue::Array(Some(array)))
            }
            b => Err(RespError::InvalidType(b as char)),
        }
    }

    /// Serialize a RESP value to bytes
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RespValue::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RespValue::Error(s) => format!("-{}\r\n", s).into_bytes(),
            RespValue::Integer(i) => format!(":{}\r\n", i).into_bytes(),
            RespValue::BulkString(None) => b"$-1\r\n".to_vec(),
            RespValue::BulkString(Some(bytes)) => {
                let mut result = format!("${}\r\n", bytes.len()).into_bytes();
                result.extend_from_slice(bytes);
                result.extend_from_slice(b"\r\n");
                result
            }
            RespValue::Array(None) => b"*-1\r\n".to_vec(),
            RespValue::Array(Some(values)) => {
                let mut result = format!("*{}\r\n", values.len()).into_bytes();
                for value in values {
                    result.extend_from_slice(&value.serialize());
                }
                result
            }
        }
    }

    /// Convert to string if possible
    pub fn as_str(&self) -> Result<&str, RespError> {
        match self {
            RespValue::SimpleString(s) => Ok(s),
            RespValue::BulkString(Some(bytes)) => {
                std::str::from_utf8(bytes).map_err(RespError::from)
            }
            _ => Err(RespError::InvalidFormat("Not a string".into())),
        }
    }

    /// Convert to bytes if possible
    pub fn as_bytes(&self) -> Result<&[u8], RespError> {
        match self {
            RespValue::BulkString(Some(bytes)) => Ok(bytes),
            RespValue::SimpleString(s) => Ok(s.as_bytes()),
            _ => Err(RespError::InvalidFormat("Not a bulk string".into())),
        }
    }

    /// Convert to array if possible
    pub fn into_array(self) -> Result<Vec<RespValue>, RespError> {
        match self {
            RespValue::Array(Some(arr)) => Ok(arr),
            _ => Err(RespError::InvalidFormat("Not an array".into())),
        }
    }
}

/// Read a line from the cursor (until \r\n)
fn read_line(src: &mut Cursor<&[u8]>) -> Result<String, RespError> {
    let start = src.position() as usize;
    let slice = src.get_ref();

    for i in start..slice.len() - 1 {
        if slice[i] == b'\r' && slice[i + 1] == b'\n' {
            let line = &slice[start..i];
            src.set_position((i + 2) as u64);
            return Ok(std::str::from_utf8(line)?.to_string());
        }
    }

    Err(RespError::Incomplete)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        let data = b"+OK\r\n";
        let mut cursor = Cursor::new(&data[..]);
        let value = RespValue::parse(&mut cursor).unwrap();
        assert_eq!(value, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_parse_error() {
        let data = b"-Error message\r\n";
        let mut cursor = Cursor::new(&data[..]);
        let value = RespValue::parse(&mut cursor).unwrap();
        assert_eq!(value, RespValue::Error("Error message".to_string()));
    }

    #[test]
    fn test_parse_integer() {
        let data = b":1000\r\n";
        let mut cursor = Cursor::new(&data[..]);
        let value = RespValue::parse(&mut cursor).unwrap();
        assert_eq!(value, RespValue::Integer(1000));
    }

    #[test]
    fn test_parse_bulk_string() {
        let data = b"$6\r\nfoobar\r\n";
        let mut cursor = Cursor::new(&data[..]);
        let value = RespValue::parse(&mut cursor).unwrap();
        assert_eq!(
            value,
            RespValue::BulkString(Some(b"foobar".to_vec()))
        );
    }

    #[test]
    fn test_parse_null_bulk_string() {
        let data = b"$-1\r\n";
        let mut cursor = Cursor::new(&data[..]);
        let value = RespValue::parse(&mut cursor).unwrap();
        assert_eq!(value, RespValue::BulkString(None));
    }

    #[test]
    fn test_parse_array() {
        let data = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let mut cursor = Cursor::new(&data[..]);
        let value = RespValue::parse(&mut cursor).unwrap();
        assert_eq!(
            value,
            RespValue::Array(Some(vec![
                RespValue::BulkString(Some(b"foo".to_vec())),
                RespValue::BulkString(Some(b"bar".to_vec())),
            ]))
        );
    }

    #[test]
    fn test_serialize_simple_string() {
        let value = RespValue::SimpleString("OK".to_string());
        assert_eq!(value.serialize(), b"+OK\r\n");
    }

    #[test]
    fn test_serialize_bulk_string() {
        let value = RespValue::BulkString(Some(b"foobar".to_vec()));
        assert_eq!(value.serialize(), b"$6\r\nfoobar\r\n");
    }

    #[test]
    fn test_serialize_array() {
        let value = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"foo".to_vec())),
            RespValue::BulkString(Some(b"bar".to_vec())),
        ]));
        assert_eq!(value.serialize(), b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
    }

    #[test]
    fn test_incomplete_message() {
        let data = b"+OK\r";
        let mut cursor = Cursor::new(&data[..]);
        let result = RespValue::parse(&mut cursor);
        assert!(matches!(result, Err(RespError::Incomplete)));
    }
}
