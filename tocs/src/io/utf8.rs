use core::num::NonZeroUsize;

use super::{PaddedByteStr, PaddedByteString, PaddedStringError, PaddedStringReadError};

/// UTF-8 string that's null-terminated and padded to a given length
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PaddedUtf8String<const PADDING: usize> {
    s: ecow::EcoString,
}

/// Reference to a UTF-8 string that's null-terminated and padded to a given length
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PaddedUtf8Str<'a, const PADDING: usize> {
    s: &'a str,
}

fn check_str(s: &str) -> Result<(), PaddedStringError> {
    for x in s.chars() {
        if u32::from(x) == 0u32 {
            return Err(PaddedStringError::ContainsNull);
        }
    }
    Ok(())
}

impl<const PADDING: usize> PaddedUtf8String<PADDING> {
    const P: NonZeroUsize = match NonZeroUsize::new(PADDING) {
        Some(p) => p,
        None => panic!("PADDING = 0 does not make sense"),
    };

    pub fn new(str_excluding_terminator_or_padding: &str) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        check_str(str_excluding_terminator_or_padding)?;
        Ok(PaddedUtf8String {
            s: str_excluding_terminator_or_padding.into(),
        })
    }

    pub fn from_eco_string(str_excluding_terminator_or_padding: &ecow::EcoString) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        check_str(str_excluding_terminator_or_padding.as_str())?;
        Ok(PaddedUtf8String {
            s: str_excluding_terminator_or_padding.clone(),
        })
    }

    pub fn from_utf8(bytes_excluding_terminator_or_padding: &[u8]) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        let s = std::str::from_utf8(bytes_excluding_terminator_or_padding).unwrap();
        check_str(s)?;
        Ok(PaddedUtf8String { s: s.into() })
    }

    pub fn from_utf8_lossy(bytes: PaddedByteStr<'_, PADDING>) -> Self {
        _ = Self::P;
        let s = String::from_utf8_lossy(bytes.as_bytes_excluding_terminator_and_padding());
        PaddedUtf8String { s: s.into() }
    }

    pub fn as_padded_utf8_str(&self) -> PaddedUtf8Str<'_, PADDING> {
        _ = Self::P;
        PaddedUtf8Str { s: self.s.as_str() }
    }

    pub fn into_inner(self) -> ecow::EcoString {
        self.s
    }
}

impl<'a, const PADDING: usize> PaddedUtf8Str<'a, PADDING> {
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

    pub fn from_utf8(bytes_excluding_terminator_or_padding: &'a [u8]) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        let s = std::str::from_utf8(bytes_excluding_terminator_or_padding).map_err(PaddedStringError::Utf8)?;
        check_str(s)?;
        Ok(Self { s })
    }

    pub fn to_owned(self) -> PaddedUtf8String<PADDING> {
        _ = Self::P;
        PaddedUtf8String { s: self.s.into() }
    }

    pub fn as_padded_byte_str(self) -> PaddedByteStr<'a, PADDING> {
        self.into()
    }

    pub fn as_str(&self) -> &'a str {
        self.s
    }
}

impl<const PADDING: usize> PaddedUtf8String<PADDING> {
    pub fn read(f: impl std::io::Read) -> Result<Self, PaddedStringReadError> {
        PaddedByteString::read(f).and_then(|bytes| Self::try_from(bytes).map_err(|err| err.try_into().unwrap()))
    }
}

impl<'a, const PADDING: usize> PaddedUtf8Str<'a, PADDING> {
    pub fn read_from_slice(slice: &'a [u8]) -> Result<(Self, &'a [u8]), PaddedStringReadError> {
        let (bytes, rest) = PaddedByteStr::read_from_slice(slice)?;
        let utf8 = Self::try_from(bytes).map_err(|err| PaddedStringReadError::try_from(err).unwrap())?;
        Ok((utf8, rest))
    }
}

impl<'a, const PADDING: usize> PaddedUtf8Str<'a, PADDING> {
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

impl<const PADDING: usize> PaddedUtf8String<PADDING> {
    pub fn as_str(&self) -> &str {
        self.s.as_str()
    }

    pub fn len_of_padding(&self) -> usize {
        self.as_padded_utf8_str().len_of_padding()
    }

    pub fn len_with_padding(&self) -> NonZeroUsize {
        self.as_padded_utf8_str().len_with_padding()
    }

    pub fn len_excluding_terminator_and_padding(&self) -> usize {
        self.as_padded_utf8_str().len_excluding_terminator_and_padding()
    }

    pub fn as_padded_byte_str(&self) -> PaddedByteStr<'_, PADDING> {
        self.as_padded_utf8_str().as_padded_byte_str()
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &[u8] {
        self.as_padded_utf8_str().as_bytes_excluding_terminator_and_padding()
    }

    pub fn write_to(&self, f: impl std::io::Write) -> std::io::Result<usize> {
        self.as_padded_byte_str().write_to(f)
    }
}

impl<const PADDING: usize> std::fmt::Display for PaddedUtf8String<PADDING> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.s.fmt(f)
    }
}

impl<'a, const PADDING: usize> TryFrom<PaddedByteStr<'a, PADDING>> for PaddedUtf8Str<'a, PADDING> {
    type Error = PaddedStringError;

    fn try_from(bytes: PaddedByteStr<'a, PADDING>) -> Result<Self, Self::Error> {
        Self::from_utf8(bytes.as_bytes_excluding_terminator_and_padding())
    }
}

impl<'a, const PADDING: usize> From<PaddedUtf8Str<'a, PADDING>> for PaddedByteStr<'a, PADDING> {
    fn from(utf8: PaddedUtf8Str<'a, PADDING>) -> PaddedByteStr<'a, PADDING> {
        PaddedByteStr::new_unchecked(utf8.as_bytes_excluding_terminator_and_padding())
    }
}

impl<const PADDING: usize> TryFrom<PaddedByteString<PADDING>> for PaddedUtf8String<PADDING> {
    type Error = PaddedStringError;

    fn try_from(bytes: PaddedByteString<PADDING>) -> Result<Self, Self::Error> {
        Self::from_utf8(bytes.as_bytes_excluding_terminator_and_padding())
    }
}
