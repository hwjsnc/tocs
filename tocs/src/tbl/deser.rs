use crate::io::{MemReader, PaddedAsciiStr, PaddedByteStr, PaddedUtf8Str, PaddedUtf8String};

use super::{AtomicType, AtomicValue, Attribute, Header};
use crate::tbl::schema::HeaderRef;

use indexmap::IndexMap;

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
    Utf8(std::str::Utf8Error, ecow::EcoString),
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

struct TblReader<'a> {
    inner: MemReader<'a>,
    strict: bool,
    warning_logger: Option<&'a mut WarningLogger<DeserializeError>>,
}

impl<'a> TblReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            inner: MemReader::new(data),
            strict: true,
            warning_logger: None,
        }
    }

    pub fn new_lax(data: &'a [u8], logger: &'a mut WarningLogger<DeserializeError>) -> Self {
        Self {
            inner: MemReader::new(data),
            strict: false,
            warning_logger: Some(logger),
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
    #[inline(always)]
    fn log_warning(&mut self, warning: DeserializeError) {
        if let Some(logger) = self.warning_logger.as_mut() {
            logger.log(warning)
        }
    }

    #[inline(always)]
    fn warning_index(&self) -> usize {
        self.warning_logger.as_ref().map(|logger| logger.len()).unwrap_or(0)
    }

    #[inline(always)]
    fn add_warning_context(&mut self, warnings_since: usize, context: DeserializeErrorPosition) {
        if let Some(logger) = self.warning_logger.as_mut() {
            logger.add_context_for_all_since(warnings_since, context);
        }
    }

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
                    let s = String::from_utf8_lossy(bytes.as_bytes_excluding_terminator_and_padding()).into();
                    self.error(DeserializeErrorDetail::Utf8(err, s)).with_context(DeserializeErrorPosition::Value(AtomicType::CUtf8))
                }
                PaddedStringError::ContainsNull => unreachable!(),
                PaddedStringError::Ascii => unreachable!(),
            }
        })
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
                        let s = String::from_utf8_lossy(bytes.as_bytes_excluding_terminator_and_padding()).into();
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
    /// Read tbl header for Trails of Cold Steel (no list of entry headers at the start)
    pub fn read_tbl_header_v1(&mut self) -> Result<u16, DeserializeError> {
        self.read_u16_le().map_err(|err| err.with_context(DeserializeErrorPosition::TblHeader))
    }

    /// Read tbl header for Trails of Cold Steel 2 - 5 (includes list of entry headers at the start)
    pub fn read_tbl_header_v2(&mut self) -> Result<(u16, IndexMap<Header, u32>), DeserializeError> {
        let number_of_entries = self.read_u16_le().map_err(|err| err.with_context(DeserializeErrorPosition::TblHeader))?;
        let number_of_headers = self.read_u32_le().map_err(|err| err.with_context(DeserializeErrorPosition::TblHeader))?;
        let mut entry_counts = IndexMap::with_capacity(number_of_entries.try_into().unwrap());
        for i in 0..number_of_headers {
            let ctx = DeserializeErrorPosition::TblHeaderEntry { entry_number: i };
            let hdr = self.read_header_name().map_err(|err| err.with_context(ctx.clone()))?;
            let count = self.read_u32_le().map_err(|err| err.with_context(ctx))?;
        }
        Ok((number_of_entries, entry_counts))
    }

    pub fn read_value(&mut self, t: AtomicType) -> Result<AtomicValue, DeserializeError> {
        match t {
            AtomicType::I8 => self.read_i8().map(AtomicValue::I8),
            AtomicType::I16 => self.read_i16_le().map(AtomicValue::I16),
            AtomicType::I32 => self.read_i32_le().map(AtomicValue::I32),
            AtomicType::U8 => self.read_u8().map(AtomicValue::U8),
            AtomicType::U16 => self.read_u16_le().map(AtomicValue::U16),
            AtomicType::U32 => self.read_u32_le().map(AtomicValue::U32),
            AtomicType::F32 => self.read_f32_le().map(AtomicValue::F32),
            AtomicType::CUtf8 => match self.read_c_utf8_str() {
                Ok(utf8) => Ok(AtomicValue::CUtf8(utf8.as_str().into())),
                Err(err) => match (self.strict, &err.detail) {
                    (false, DeserializeErrorDetail::Utf8(e, s)) => {
                        let ret = s.clone();
                        self.log_warning(err);
                        Ok(AtomicValue::CUtf8(ret))
                    }
                    _ => Err(err),
                },
            },
            AtomicType::X(n) => self.read_bytes(n).map(|slice| AtomicValue::X(slice.into())),
        }
    }

    pub fn read_tbl_entry_data(&mut self, schema: &IndexMap<Attribute, AtomicType>) -> Result<IndexMap<Attribute, AtomicValue>, DeserializeError> {
        let mut values = IndexMap::with_capacity(schema.len());
        for (i, (attr_name, attr_type)) in schema.iter().enumerate() {
            let wi = self.warning_index();
            let val = self.read_value(*attr_type).map_err(|err| err.with_context(DeserializeErrorPosition::EntryData))?;
            self.add_warning_context(wi, DeserializeErrorPosition::EntryData);
        }
        Ok(values)
    }
}
