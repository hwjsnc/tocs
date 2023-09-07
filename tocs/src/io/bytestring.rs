use super::{read_byte, PaddedStringError, PaddedStringReadError};
use core::num::NonZeroUsize;
use smallvec::SmallVec;

/// A string of non-null bytes, followed by a terminating null-byte and padded (with null bytes) to a multiple of a given length.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaddedByteString<const PADDING: usize> {
    /// Bytes without null terminator or padding.
    bytes: smallvec::SmallVec<[u8; 64]>, // TODO: replace with ecow::EcoBytes or something
}

/// Reference to a string of non-null bytes, which when serializing is followed by a terminating null-byte and padded (with null-bytes) to a multiple of a given length.
///
/// The underlying buffer is not necessarily null-terminated and padded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaddedByteStr<'a, const PADDING: usize> {
    /// Bytes without null terminator or padding
    bytes: &'a [u8],
}

impl<const PADDING: usize> PaddedByteString<PADDING> {
    const P: NonZeroUsize = match NonZeroUsize::new(PADDING) {
        Some(p) => p,
        None => panic!("PADDING = 0 does not make sense"),
    };

    pub fn new(bytes_without_terminator_or_padding: &[u8]) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        PaddedByteStr::new(bytes_without_terminator_or_padding).map(|s| s.to_owned())
    }

    pub fn as_padded_byte_str(&self) -> PaddedByteStr<'_, PADDING> {
        _ = Self::P;
        PaddedByteStr { bytes: self.bytes.as_slice() }
    }

    /// Read `PaddedByteString` from a file
    pub fn read(mut f: impl std::io::Read) -> Result<Self, PaddedStringReadError> {
        let mut bytes: SmallVec<[u8; 64]> = SmallVec::new();
        let mut i = 0;
        loop {
            let x = read_byte(&mut f).map_err(PaddedStringReadError::Io)?;
            i += 1;
            if x == 0 {
                break;
            } else {
                bytes.push(x);
            }
        }

        // read padding
        while (i % PADDING) != 0 {
            match read_byte(&mut f) {
                Ok(0) => {}
                Ok(x) => {
                    return Err(PaddedStringReadError::Padding(x));
                }
                Err(e) => {
                    return Err(PaddedStringReadError::Io(e));
                }
            }
            i += 1;
        }

        Ok(Self { bytes })
    }
}

impl<'a, const PADDING: usize> PaddedByteStr<'a, PADDING> {
    const P: NonZeroUsize = match NonZeroUsize::new(PADDING) {
        Some(p) => p,
        None => panic!("PADDING = 0 does not make sense"),
    };

    pub fn new(bytes_without_terminator_or_padding: &'a [u8]) -> Result<Self, PaddedStringError> {
        _ = Self::P;
        if bytes_without_terminator_or_padding.iter().copied().all(|c| c != 0) {
            Ok(Self {
                bytes: bytes_without_terminator_or_padding,
            })
        } else {
            Err(PaddedStringError::ContainsNull)
        }
    }

    pub fn to_owned(&self) -> PaddedByteString<PADDING> {
        _ = Self::P;
        PaddedByteString { bytes: self.bytes.into() }
    }

    /// Read a `PaddedByteStr` from the given slice.
    ///
    /// On success, returns the `PaddedByteStr` and any remaining data after the padding.
    pub fn read_from_slice(slice: &'a [u8]) -> Result<(Self, &'a [u8]), PaddedStringReadError> {
        // find first null byte
        let end = slice.iter().copied().position(|x| x == 0).ok_or(PaddedStringReadError::Io(std::io::ErrorKind::UnexpectedEof.into()))?;

        // compute necessary padding
        let padding_len = if end % Self::P == 0 { 0 } else { PADDING - (end % Self::P) };
        let end_of_padding = end + padding_len;

        // split slice into bytes (before null), padding (including terminator), rest
        let bytes = &slice[..end];
        let padding = slice.get(end..end_of_padding).ok_or(PaddedStringReadError::Io(std::io::ErrorKind::UnexpectedEof.into()))?;
        let rest = &slice[end_of_padding..];

        // check that padding is all zero
        if let Some(x) = padding.iter().copied().find(|x| *x != 0) {
            Err(PaddedStringReadError::Padding(x))
        } else {
            Ok((Self { bytes }, rest))
        }
    }
}

impl<'a, const PADDING: usize> PaddedByteStr<'a, PADDING> {
    pub fn len_of_padding(&self) -> usize {
        let len_with_terminator = self.bytes.len() + 1;
        let r = len_with_terminator % Self::P;
        if r == 0 {
            0
        } else {
            PADDING - r
        }
    }

    pub fn len_with_padding(&self) -> NonZeroUsize {
        NonZeroUsize::new(1 + self.bytes.len() + self.len_of_padding()).unwrap()
    }

    pub fn len_excluding_terminator_and_padding(&self) -> usize {
        self.bytes.len()
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &'a [u8] {
        self.bytes
    }

    pub fn write_to(&self, mut f: impl std::io::Write) -> std::io::Result<usize> {
        f.write_all(self.bytes)?;
        f.write_all(&[0u8; PADDING][..1 + self.len_of_padding()])?;
        Ok(self.len_with_padding().get())
    }

    pub fn to_str(self) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self.bytes)
    }

    pub fn to_string_lossy(self) -> std::borrow::Cow<'a, str> {
        String::from_utf8_lossy(self.bytes)
    }
}

impl<const PADDING: usize> PaddedByteString<PADDING> {
    pub fn len_of_padding(&self) -> usize {
        self.as_padded_byte_str().len_of_padding()
    }

    pub fn len_with_padding(&self) -> NonZeroUsize {
        self.as_padded_byte_str().len_with_padding()
    }

    pub fn len_excluding_terminator_and_padding(&self) -> usize {
        self.as_padded_byte_str().len_excluding_terminator_and_padding()
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &[u8] {
        self.as_padded_byte_str().as_bytes_excluding_terminator_and_padding()
    }

    pub fn write_to(&self, f: impl std::io::Write) -> std::io::Result<usize> {
        self.as_padded_byte_str().write_to(f)
    }
}
