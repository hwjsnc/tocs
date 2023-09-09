use core::num::NonZeroUsize;

use super::{PaddedByteStr, PaddedByteString, PaddedStringError, PaddedStringReadError};

/// ASCII string that's null-terminated and padded to a given length
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PaddedAsciiString<const PADDING: usize> {
    s: ecow::EcoString,
}

/// Reference to an ASCII string that's null-terminated and padded to a given length
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PaddedAsciiStr<'a, const PADDING: usize> {
    s: &'a str,
}

fn check_bytes(slice: &[u8]) -> Result<(), PaddedStringError> {
    for x in slice.iter().copied() {
        if x == 0 {
            return Err(PaddedStringError::ContainsNull);
        }
        if !x.is_ascii() {
            return Err(PaddedStringError::Ascii);
        }
    }
    Ok(())
}

fn check_str(s: &str) -> Result<(), PaddedStringError> {
    check_bytes(s.as_bytes())?; // ASCII is exactly the set of chars that UTF-8 encodes as one byte
    Ok(())
}

impl<const PADDING: usize> PaddedAsciiString<PADDING> {
    const P: NonZeroUsize = match NonZeroUsize::new(PADDING) {
        Some(p) => p,
        None => panic!("PADDING = 0 does not make sense"),
    };

    pub fn new(str_excluding_terminator_or_padding: &str) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        check_str(str_excluding_terminator_or_padding)?;
        Ok(PaddedAsciiString {
            s: str_excluding_terminator_or_padding.into(),
        })
    }

    pub fn from_eco_string(str_excluding_terminator_or_padding: &ecow::EcoString) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        check_str(str_excluding_terminator_or_padding.as_str())?;
        Ok(PaddedAsciiString {
            s: str_excluding_terminator_or_padding.clone(),
        })
    }

    pub fn from_ascii(bytes_excluding_terminator_or_padding: &[u8]) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        check_bytes(bytes_excluding_terminator_or_padding)?;
        // We iterate through the data twice to avoid `unsafe`
        let s = std::str::from_utf8(bytes_excluding_terminator_or_padding).unwrap();
        Ok(PaddedAsciiString { s: s.into() })
    }

    pub fn as_padded_ascii_str(&self) -> PaddedAsciiStr<'_, PADDING> {
        _ = Self::P;
        PaddedAsciiStr { s: self.s.as_str() }
    }
}

impl<'a, const PADDING: usize> PaddedAsciiStr<'a, PADDING> {
    const P: NonZeroUsize = match NonZeroUsize::new(PADDING) {
        Some(p) => p,
        None => panic!("PADDING = 0 does not make sense"),
    };

    pub fn new(str_excluding_terminator_or_padding: &'a str) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        check_str(str_excluding_terminator_or_padding)?;
        Ok(Self {
            s: str_excluding_terminator_or_padding,
        })
    }

    pub fn from_ascii(bytes_excluding_terminator_or_padding: &'a [u8]) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        check_bytes(bytes_excluding_terminator_or_padding)?;
        // We iterate through the data twice to avoid `unsafe`
        let s = std::str::from_utf8(bytes_excluding_terminator_or_padding).unwrap();
        Ok(Self { s })
    }

    pub fn to_owned(&self) -> PaddedAsciiString<PADDING> {
        _ = Self::P;
        PaddedAsciiString { s: self.s.into() }
    }

    pub fn as_padded_byte_str(self) -> PaddedByteStr<'a, PADDING> {
        self.into()
    }
}

impl<const PADDING: usize> PaddedAsciiString<PADDING> {
    pub fn read(f: impl std::io::Read) -> Result<Self, PaddedStringReadError> {
        PaddedByteString::read(f).and_then(|bytes| Self::try_from(bytes).map_err(|_| PaddedStringReadError::Ascii))
    }
}

impl<'a, const PADDING: usize> PaddedAsciiStr<'a, PADDING> {
    pub fn read_from_slice(slice: &'a [u8]) -> Result<(Self, &'a [u8]), PaddedStringReadError> {
        let (bytes, rest) = PaddedByteStr::read_from_slice(slice)?;
        let ascii = Self::try_from(bytes).map_err(|_| PaddedStringReadError::Ascii)?;
        Ok((ascii, rest))
    }
}

impl<'a, const PADDING: usize> PaddedAsciiStr<'a, PADDING> {
    pub fn to_str(self) -> &'a str {
        self.s
    }

    pub fn len_of_padding(&self) -> usize {
        self.as_padded_byte_str().len_of_padding()
    }

    pub fn len_with_padding(&self) -> NonZeroUsize {
        self.as_padded_byte_str().len_with_padding()
    }

    pub fn len_excluding_terminator_and_padding(&self) -> usize {
        self.as_padded_byte_str().len_excluding_terminator_and_padding()
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &'a [u8] {
        self.s.as_bytes()
    }

    pub fn write_to(&self, f: impl std::io::Write) -> std::io::Result<usize> {
        self.as_padded_byte_str().write_to(f)
    }
}

impl<const PADDING: usize> PaddedAsciiString<PADDING> {
    pub fn as_str(&self) -> &str {
        self.s.as_str()
    }

    pub fn len_of_padding(&self) -> usize {
        self.as_padded_ascii_str().len_of_padding()
    }

    pub fn len_with_padding(&self) -> NonZeroUsize {
        self.as_padded_ascii_str().len_with_padding()
    }

    pub fn len_excluding_terminator_and_padding(&self) -> usize {
        self.as_padded_ascii_str().len_excluding_terminator_and_padding()
    }

    pub fn as_padded_byte_str(&self) -> PaddedByteStr<'_, PADDING> {
        self.as_padded_ascii_str().as_padded_byte_str()
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &[u8] {
        self.as_padded_ascii_str().as_bytes_excluding_terminator_and_padding()
    }

    pub fn write_to(&self, f: impl std::io::Write) -> std::io::Result<usize> {
        self.as_padded_byte_str().write_to(f)
    }
}

impl<const PADDING: usize> std::fmt::Display for PaddedAsciiString<PADDING> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.s.fmt(f)
    }
}

impl<'a, const PADDING: usize> TryFrom<PaddedByteStr<'a, PADDING>> for PaddedAsciiStr<'a, PADDING> {
    type Error = PaddedStringError;

    fn try_from(bytes: PaddedByteStr<'a, PADDING>) -> Result<Self, Self::Error> {
        Self::from_ascii(bytes.as_bytes_excluding_terminator_and_padding())
    }
}

impl<'a, const PADDING: usize> From<PaddedAsciiStr<'a, PADDING>> for PaddedByteStr<'a, PADDING> {
    fn from(ascii: PaddedAsciiStr<'a, PADDING>) -> PaddedByteStr<'a, PADDING> {
        PaddedByteStr::new_unchecked(ascii.as_bytes_excluding_terminator_and_padding())
    }
}

impl<const PADDING: usize> TryFrom<PaddedByteString<PADDING>> for PaddedAsciiString<PADDING> {
    type Error = PaddedStringError;

    fn try_from(bytes: PaddedByteString<PADDING>) -> Result<Self, Self::Error> {
        Self::from_ascii(bytes.as_bytes_excluding_terminator_and_padding())
    }
}

impl<'a, const PADDING: usize> PartialEq<PaddedAsciiString<PADDING>> for PaddedAsciiStr<'a, PADDING> {
    fn eq(&self, other: &PaddedAsciiString<PADDING>) -> bool {
        self.s == other.s
    }
}

impl<'a, const PADDING: usize> PartialEq<PaddedAsciiStr<'a, PADDING>> for PaddedAsciiString<PADDING> {
    fn eq(&self, other: &PaddedAsciiStr<'a, PADDING>) -> bool {
        self.s == other.s
    }
}

impl<'a, const PADDING: usize> indexmap::Equivalent<PaddedAsciiStr<'a, PADDING>> for PaddedAsciiString<PADDING> {
    fn equivalent(&self, other: &PaddedAsciiStr<'a, PADDING>) -> bool {
        self == other
    }
}
