mod ascii;
mod bytestring;

// TODO: ascii name (for pka/pkg)
pub use ascii::{PaddedAsciiStr, PaddedAsciiString};
pub use bytestring::{PaddedByteStr, PaddedByteString};

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

/// A null-terminated UTF-8 string, padded to a multiple of a certain length with null bytes
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaddedUtf8String<const PADDING: usize> {
    bytes: PaddedByteString<PADDING>,
}

impl<const PADDING: usize> PaddedUtf8String<PADDING> {
    #[allow(dead_code)]
    pub fn new(str: &str) -> Result<Self, PaddedStringError> {
        PaddedByteString::new(str.as_bytes()).map(|bytes| Self { bytes })
    }

    #[allow(dead_code)]
    pub fn len_with_padding(&self) -> usize {
        self.bytes.len_with_padding().get()
    }

    #[allow(dead_code)]
    pub fn write_to<F: std::io::Write>(&self, f: &mut F) -> std::io::Result<usize> {
        self.bytes.write_to(f)
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &[u8] {
        self.bytes.as_bytes_excluding_terminator_and_padding()
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes_excluding_terminator_and_padding()).expect("creation is only possible with valid UTF-8 strings")
    }
}

impl<const PADDING: usize> std::fmt::Display for PaddedUtf8String<PADDING> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.as_str().fmt(f)
    }
}

impl<const PADDING: usize> TryFrom<PaddedByteString<PADDING>> for PaddedUtf8String<PADDING> {
    type Error = PaddedStringError;

    fn try_from(bytes: PaddedByteString<PADDING>) -> Result<Self, Self::Error> {
        let _ = std::str::from_utf8(bytes.as_bytes_excluding_terminator_and_padding()).map_err(PaddedStringError::Utf8)?;
        Ok(Self { bytes })
    }
}
