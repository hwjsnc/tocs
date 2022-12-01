use smallvec::SmallVec;
use smol_str::SmolStr;

pub fn read_byte<F: std::io::Read>(f: &mut F) -> Result<u8, std::io::Error> {
    let a: [u8; 1] = read_bytes(f)?;
    Ok(a[0])
}

pub fn read_bytes<const N: usize, F: std::io::Read>(f: &mut F) -> Result<[u8; N], std::io::Error> {
    let mut buf: [u8; N] = [0; N];
    f.read_exact(&mut buf)?;
    Ok(buf)
}

pub enum ReadUtf8Error {
    IO(std::io::Error),
    Utf8(std::str::Utf8Error),
    UnexpectedPadding(u8),
}

pub fn read_cutf8_padded<F: std::io::Read>(f: &mut F, chunk_size: usize) -> Result<SmolStr, ReadUtf8Error> {
    let mut data: SmallVec<[u8; 64]> = SmallVec::new();
    let mut i = 0;
    loop {
        let x = read_byte(f).map_err(ReadUtf8Error::IO)?;
        i += 1;
        if x == 0 {
            break;
        } else {
            data.push(x);
        }
    }

    // read padding until we hit the chunk size
    while (i % chunk_size) != 0 {
        match read_byte(f) {
            Ok(0) => {}
            Ok(x) => {
                return Err(ReadUtf8Error::UnexpectedPadding(x));
            }
            Err(e) => {
                return Err(ReadUtf8Error::IO(e));
            }
        }
        i += 1;
    }

    // check utf-8
    std::str::from_utf8(&data).map(SmolStr::from).map_err(ReadUtf8Error::Utf8)
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

/// A string of non-null bytes, followed by a terminating null-byte and padded (with null bytes) to a multiple of a certain length
#[derive(Clone, Debug)]
pub struct PaddedByteString {
    bytes: smallvec::SmallVec<[u8; 64]>, // bytestring excluding null terminator and padding
    padding: usize,
}

impl PaddedByteString {
    pub fn new(bytes: &[u8], padding: usize) -> Result<Self, PaddedStringError> {
        assert!(padding >= 1);
        if bytes.iter().copied().any(|c| c == 0) {
            Err(PaddedStringError::ContainsNull)
        } else {
            Ok(Self {
                bytes: bytes.as_ref().into(),
                padding,
            })
        }
    }

    pub fn len_with_padding(&self) -> usize {
        let (n, r) = num_integer::div_rem(self.bytes.len() + 1, self.padding);
        if r == 0 {
            n * self.padding
        } else {
            (n + 1) * self.padding
        }
    }

    pub fn write_to<F: std::io::Write>(&self, f: &mut F) -> std::io::Result<usize> {
        let mut n = f.write(&self.bytes)?;
        n += f.write(&[0])?;
        loop {
            n += f.write(&[0])?;
            if n % self.padding == 0 {
                assert_eq!(n, self.len_with_padding());
                return Ok(n);
            }
        }
    }

    pub fn padding(&self) -> usize {
        self.padding
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &[u8] {
        &self.bytes
    }
}

/// A null-terminated ASCII string, padded to a multiple of a certain length with null bytes
#[derive(Clone, Debug)]
pub struct PaddedAsciiString {
    bytes: PaddedByteString,
}

impl PaddedAsciiString {
    pub fn new(str: &str, padding: usize) -> Result<Self, PaddedStringError> {
        assert!(padding >= 1);
        if str.is_ascii() {
            PaddedByteString::new(str.as_bytes(), padding).map(|bytes| Self { bytes })
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

    pub fn padding(&self) -> usize {
        self.bytes.padding()
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

impl std::fmt::Display for PaddedAsciiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.as_str().fmt(f)
    }
}
