mod ascii;
mod bytestring;
mod utf8;

// TODO: ascii name (for pka/pkg)
pub use ascii::{PaddedAsciiStr, PaddedAsciiString};
pub use bytestring::{PaddedByteStr, PaddedByteString};
pub use utf8::{PaddedUtf8Str, PaddedUtf8String};

// TODO: slice reader and std::io::Read reader

pub fn read_byte(f: impl std::io::Read) -> Result<u8, std::io::Error> {
    let a: [u8; 1] = read_bytes(f)?;
    Ok(a[0])
}

pub fn read_bytes<const N: usize>(mut f: impl std::io::Read) -> Result<[u8; N], std::io::Error> {
    let mut buf: [u8; N] = [0; N];
    f.read_exact(&mut buf)?;
    Ok(buf)
}

#[derive(Clone, Debug)]
pub enum PaddedStringError {
    ContainsNull,
    Ascii,
    Utf8(std::str::Utf8Error),
}

impl std::fmt::Display for PaddedStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PaddedStringError::ContainsNull => write!(f, "bytes contain null byte"),
            PaddedStringError::Ascii => write!(f, "non-ASCII characters present"),
            PaddedStringError::Utf8(e) => write!(f, "UTF-8 Error: {e}"),
        }
    }
}

impl std::error::Error for PaddedStringError {}

#[derive(Debug)]
pub enum PaddedStringReadError {
    Io(std::io::Error),
    Padding(u8),
    Utf8(std::str::Utf8Error),
    Ascii,
}

impl std::fmt::Display for PaddedStringReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PaddedStringReadError::Io(e) => write!(f, "IO error: {e}"),
            PaddedStringReadError::Padding(b) => write!(f, "Unexpected padding: {b} (0x{b:x})"),
            PaddedStringReadError::Ascii => write!(f, "non-ASCII characters present"),
            PaddedStringReadError::Utf8(e) => write!(f, "UTF-8 error: {e}"),
        }
    }
}

impl std::error::Error for PaddedStringReadError {}

impl TryFrom<PaddedStringError> for PaddedStringReadError {
    type Error = ();

    fn try_from(err: PaddedStringError) -> Result<PaddedStringReadError, ()> {
        match err {
            PaddedStringError::ContainsNull => Err(()),
            PaddedStringError::Ascii => Ok(PaddedStringReadError::Ascii),
            PaddedStringError::Utf8(e) => Ok(PaddedStringReadError::Utf8(e)),
        }
    }
}

impl From<std::io::Error> for PaddedStringReadError {
    fn from(err: std::io::Error) -> PaddedStringReadError {
        PaddedStringReadError::Io(err)
    }
}
