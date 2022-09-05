/// SJisString is an immutable S-JIS encoded string with conversion from/to bytes as well as UTF-8.
/// Should only be used for "short" strings (a few kB).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SJisString {
    data: std::vec::Vec<u8>,
    utf8: String, // the same data (losslessly) converted to UTF-8, for display purposes
}

impl TryFrom<&str> for SJisString {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let (data, encoding, fail) = encoding_rs::SHIFT_JIS.encode(s);
        if fail {
            return Err(());
        }
        assert_eq!(encoding, encoding_rs::SHIFT_JIS);
        Ok(Self {
            data: data.into_owned(),
            utf8: s.to_string(),
        })
    }
}

impl std::fmt::Display for SJisString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.utf8)
    }
}

impl SJisString {
    /// Length of the Shift-JIS encoded string in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn from_s_jis(bytes: &[u8]) -> Result<Self, ()> {
        encoding_rs::SHIFT_JIS.decode_without_bom_handling_and_without_replacement(bytes).ok_or(()).map(|s| Self {
            data: bytes.to_vec(),
            utf8: s.into_owned(),
        })
    }

    pub fn from_s_jis_lossy(bytes: &[u8]) -> Self {
        let (s, _something_was_replaced) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(bytes);
        Self {
            data: bytes.to_vec(),
            utf8: s.into_owned(),
        }
    }

    pub fn from_utf8_lossy(s: &str) -> Self {
        let (data, encoding, _something_was_replaced) = encoding_rs::SHIFT_JIS.encode(s);
        assert_eq!(encoding, encoding_rs::SHIFT_JIS);
        Self {
            data: data.into_owned(),
            utf8: s.to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let utf8 = "ヤッホー";
        let sjis = SJisString::try_from(utf8).unwrap();
        let sjis2 = SJisString::from_s_jis(sjis.as_bytes()).unwrap();
        assert_eq!(sjis2.to_string(), utf8);
        assert_eq!(sjis.as_bytes(), [0x83, 0x84, 0x83, 0x62, 0x83, 0x7a, 0x81, 0x5b]);
    }

    #[test]
    fn test_2() {
        let utf8 = "hello, world!";
        let sjis = SJisString::try_from(utf8).unwrap();
        assert_eq!(sjis.as_bytes(), utf8.as_bytes());
    }
}
