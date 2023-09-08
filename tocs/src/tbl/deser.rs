use crate::io::{MemReader, PaddedAsciiStr, PaddedByteStr, PaddedUtf8Str, PaddedUtf8String};

use super::{AtomicType, Attribute, Header};
use crate::tbl::schema::HeaderRef;

// Warning Logger

pub struct WarningLogger<W> {
    ignore: bool,
    warnings: std::vec::Vec<W>,
}

impl<W> Default for WarningLogger<W> {
    fn default() -> Self {
        Self {
            ignore: false,
            warnings: std::vec::Vec::new(),
        }
    }
}

impl<W> WarningLogger<W> {
    pub fn new() -> Self {
        Self {
            ignore: false,
            warnings: std::vec::Vec::new(),
        }
    }

    fn log(&mut self, warning: W) {
        let Self { ignore, warnings } = self;
        if !*ignore {
            warnings.push(warning);
        }
    }

    #[allow(clippy::len_without_is_empty)]
    fn len(&self) -> usize {
        self.warnings.len()
    }

    fn clear(&mut self) {
        self.warnings.clear();
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    fn into_warnings(self) -> std::vec::Vec<W> {
        self.warnings
    }
}

mod detail {
    pub trait ErrorWithContext {
        type Position;
        fn add_context(&mut self, position: Self::Position);
        fn with_context(mut self, position: Self::Position) -> Self
        where
            Self: Sized,
        {
            self.add_context(position);
            self
        }
    }
}

use detail::ErrorWithContext;

impl<P: Clone, W: ErrorWithContext<Position = P>> WarningLogger<W> {
    fn add_context_for_all_since(&mut self, number: usize, context: P) {
        for w in self.warnings[number..].iter_mut() {
            w.add_context(context.clone());
        }
    }
}

// Deserialization (parsing)

#[derive(Debug)]
pub enum DeserializeErrorDetail {
    Io(std::io::Error),
    Ascii,
    Utf8(std::str::Utf8Error, String),
    SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total: u16, sum: Option<u32> },
    DuplicateHeader(Header),
    EntryHeaderNotInTblHeader { missing: Header },
    EntryCountDoesNotMatchHeaderEntry { header: Header, expected: u32, actual: usize },
    EntryDataLengthMismatch { expected: u16, actual: Option<u16> },
    TrailingData,
}

#[derive(Clone, Debug)]
pub enum DeserializeErrorPosition {
    // entry numbers should start at 0 here but will be reported as starting with 1
    Start,
    TblHeader,
    TblHeaderEntry { entry_number: u32 },
    Entry { number: u16 },
    EntryHeader,
    EntryData,
    HeaderName,
    Value(AtomicType),
    Attribute(Attribute),
    End,
}

#[derive(Debug)]
pub struct DeserializeError {
    pub detail: DeserializeErrorDetail,
    /// Seek position in the file
    pub position: usize,
    /// First element is the position of the original cause, the rest is how we got there.
    ///
    /// May be empty.
    pub context: std::vec::Vec<DeserializeErrorPosition>,
}

impl DeserializeErrorDetail {
    pub fn eof() -> Self {
        DeserializeErrorDetail::Io(std::io::ErrorKind::UnexpectedEof.into())
    }
}

impl detail::ErrorWithContext for DeserializeError {
    type Position = DeserializeErrorPosition;

    fn add_context(&mut self, position: Self::Position) {
        self.context.push(position);
    }
}

#[derive(Clone)]
struct TblReader<'a> {
    inner: MemReader<'a>,
    strict: bool,
    warnings: Option<WarningLogger<DeserializeError>>,
}

impl<'a> TblReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            inner: MemReader::new(data),
            strict: true,
        }
    }

    pub fn new_lax(data: &'a [u8]) -> Self {
        Self {
            inner: MemReader::new(data),
            strict: false,
        }
    }
}

macro_rules! impl_via_inner {
    ($f:ident, $t:ty, $at:expr) => {
        pub fn $f(&mut self) -> Result<$t, DeserializeError> {
            self.inner
                .$f()
                .ok_or_else(|| self.error(DeserializeErrorDetail::eof()).with_context(DeserializeErrorPosition::Value($at)))
        }
    };
}

impl<'a> TblReader<'a> {
    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], DeserializeError> {
        self.inner
            .read_bytes(n)
            .ok_or_else(|| self.error(DeserializeErrorDetail::eof()).with_context(DeserializeErrorPosition::Value(AtomicType::X(n))))
    }

    impl_via_inner! {read_byte, u8, AtomicType::U8}
    impl_via_inner! {read_u8, u8, AtomicType::U8}
    impl_via_inner! {read_i8, i8, AtomicType::I8}

    impl_via_inner! {read_u16_le, u16, AtomicType::U16}
    impl_via_inner! {read_u32_le, u32, AtomicType::U32}
    impl_via_inner! {read_i16_le, i16, AtomicType::I16}
    impl_via_inner! {read_i32_le, i32, AtomicType::I32}
    impl_via_inner! {read_f32_le, f32, AtomicType::F32}

    fn _read_c_byte_str(&mut self) -> Result<PaddedByteStr<'a, 1>, DeserializeError> {
        use crate::io::PaddedStringReadError;

        self.inner.read_c_byte_str().map_err(|err| {
            self.error(match err {
                PaddedStringReadError::Io(err) => DeserializeErrorDetail::Io(err),
                PaddedStringReadError::Padding(_) => unreachable!(), // tbls don't have padded strings, only null-terminated
                PaddedStringReadError::Ascii | PaddedStringReadError::Utf8(_) => unreachable!(), // we're not reading ASCII or UTF-8
            })
        })
    }

    pub fn read_c_utf8_str(&mut self) -> Result<PaddedUtf8Str<'a, 1>, DeserializeError> {
        use crate::io::PaddedStringError;

        let off = self.inner.offset();
        let bytes = self._read_c_byte_str().map_err(|e| e.with_context(DeserializeErrorPosition::Value(AtomicType::CUtf8)))?;

        bytes.try_into().map_err(|err| {
            self.inner.set_offset(off);
            match err {
                PaddedStringError::Utf8(err) => {
                    let s = String::from_utf8_lossy(bytes.as_bytes_excluding_terminator_and_padding()).into_owned();
                    self.error(DeserializeErrorDetail::Utf8(err, s)).with_context(DeserializeErrorPosition::Value(AtomicType::CUtf8))
                }
                PaddedStringError::ContainsNull => unreachable!(),
                PaddedStringError::Ascii => unreachable!(),
            }
        })
    }

    pub fn read_c_utf8_string_lossy(&mut self) -> Result<PaddedUtf8String<1>, DeserializeError> {
        self._read_c_byte_str()
            .map(PaddedUtf8String::<1>::from_utf8_lossy)
            .map_err(|err| err.with_context(DeserializeErrorPosition::Value(AtomicType::CUtf8)))
    }

    pub fn read_header_name(&mut self) -> Result<HeaderRef, DeserializeError> {
        use crate::io::PaddedStringError;

        let off = self.inner.offset();
        let bytes = self._read_c_byte_str().map_err(|e| e.with_context(DeserializeErrorPosition::HeaderName))?;

        match PaddedAsciiStr::<'_, 1>::try_from(bytes) {
            Ok(ascii) => Ok(HeaderRef::from(ascii)),
            Err(err) => {
                self.inner.set_offset(off);
                Err(match err {
                    PaddedStringError::Utf8(err) => {
                        let s = String::from_utf8_lossy(bytes.as_bytes_excluding_terminator_and_padding()).into_owned();
                        self.error(DeserializeErrorDetail::Utf8(err, s)).with_context(DeserializeErrorPosition::HeaderName)
                    }
                    PaddedStringError::ContainsNull => unreachable!(),
                    PaddedStringError::Ascii => self.error(DeserializeErrorDetail::Ascii).with_context(DeserializeErrorPosition::HeaderName),
                })
            }
        }
    }
}

impl<'a> TblReader<'a> {
    pub fn error(&self, detail: DeserializeErrorDetail) -> DeserializeError {
        DeserializeError {
            detail,
            position: self.inner.offset(),
            context: vec![],
        }
    }
}

impl<'a> TblReader<'a> {
    /*
    pub fn read_header_v1(&mut self) -> Result<u16, DeserializeError>{
        self.read_u16_le().map_err(|e| self.error(DeserializeError::)
        })
    }
    */
}
