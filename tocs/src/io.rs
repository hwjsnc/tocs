use smallvec::SmallVec;

pub fn read_byte<F: std::io::Read>(f: &mut F) -> Result<u8, std::io::Error> {
    let a: [u8; 1] = read_bytes(f)?;
    Ok(a[0])
}

pub fn read_bytes<const N: usize, F: std::io::Read>(f: &mut F) -> Result<[u8; N], std::io::Error> {
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

/// A string of non-null bytes, followed by a terminating null-byte and padded (with null bytes) to a multiple of a certain length
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] // TODO: Eq and Ord don't make sense at the moment because two different (bytes, padding) pairs may correspond to the exact same bytes => solution: make padding a compile-time associated constant and don't deal with this situation at all; for the moment, we do need Eq and Hash (for hashmaps)
pub struct PaddedByteString<const PADDING: usize> {
    bytes: smallvec::SmallVec<[u8; 64]>, // bytestring excluding null terminator and padding
}

impl<const PADDING: usize> PaddedByteString<PADDING> {
    pub fn new(bytes: &[u8]) -> Result<Self, PaddedStringError> {
        if bytes.iter().copied().any(|c| c == 0) {
            Err(PaddedStringError::ContainsNull)
        } else {
            Ok(Self { bytes: bytes.as_ref().into() })
        }
    }

    pub fn len_with_padding(&self) -> usize {
        let (n, r) = num_integer::div_rem(self.bytes.len() + 1, PADDING);
        if r == 0 {
            n * PADDING
        } else {
            (n + 1) * PADDING
        }
    }

    pub fn write_to<F: std::io::Write>(&self, f: &mut F) -> std::io::Result<usize> {
        let mut n = f.write(&self.bytes)?;
        n += f.write(&[0])?;
        loop {
            n += f.write(&[0])?;
            if n % PADDING == 0 {
                assert_eq!(n, self.len_with_padding());
                return Ok(n);
            }
        }
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &[u8] {
        &self.bytes
    }

    pub fn to_utf8_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.bytes)
    }

    pub fn to_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.bytes)
    }

    pub fn read<F: std::io::Read>(f: &mut F) -> Result<Self, PaddedStringReadError> {
        assert!(PADDING >= 1);

        let mut bytes: SmallVec<[u8; 64]> = SmallVec::new();
        let mut i = 0;
        loop {
            let x = read_byte(f).map_err(PaddedStringReadError::Io)?;
            i += 1;
            if x == 0 {
                break;
            } else {
                bytes.push(x);
            }
        }

        // read padding
        while (i % PADDING) != 0 {
            match read_byte(f) {
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

/// A null-terminated ASCII string, padded to a multiple of a certain length with null bytes
#[derive(Clone, Debug)]
pub struct PaddedAsciiString<const PADDING: usize> {
    bytes: PaddedByteString<PADDING>,
}

impl<const PADDING: usize> PaddedAsciiString<PADDING> {
    pub fn new(str: &str) -> Result<Self, PaddedStringError> {
        assert!(PADDING >= 1);
        if str.is_ascii() {
            PaddedByteString::new(str.as_bytes()).map(|bytes| Self { bytes })
        } else {
            Err(PaddedStringError::Ascii)
        }
    }

    pub fn len_with_padding(&self) -> usize {
        self.bytes.len_with_padding()
    }

    pub fn write_to<F: std::io::Write>(&self, f: &mut F) -> std::io::Result<usize> {
        self.bytes.write_to(f)
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &[u8] {
        self.bytes.as_bytes_excluding_terminator_and_padding()
    }

    pub fn as_str(&self) -> &str {
        /*
        unsafe {
            // creation is only possible with valid ASCII strings and ASCII is a subset of UTF-8
            std::str::from_utf8_unchecked(self.as_bytes_excluding_terminator_and_padding())
        }
        */
        std::str::from_utf8(self.as_bytes_excluding_terminator_and_padding()).expect("creation is only possible with valid ASCII strings")
    }
}

impl<const PADDING: usize> std::fmt::Display for PaddedAsciiString<PADDING> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.as_str().fmt(f)
    }
}
