use crate::io::{MemReader, PaddedUtf8Str, PaddedUtf8String};

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
    pub position: Option<usize>,                          // seek position in the file
    pub context: std::vec::Vec<DeserializeErrorPosition>, // first element is the position of the original cause, the rest is how we got there; may be empty
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
    ($f:ident, $result:ty) => {
        pub fn $f(&mut self) -> $result {
            self.inner.$f()
        }
    };
}

impl<'a> TblReader<'a> {
    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        self.inner.read_bytes(n)
    }

    impl_via_inner! {read_byte, Option::<u8>}
    impl_via_inner! {read_u8, Option::<u8>}
    impl_via_inner! {read_i8, Option::<i8>}

    impl_via_inner! {read_u16_le, Option::<u16>}
    impl_via_inner! {read_u32_le, Option::<u32>}
    impl_via_inner! {read_i16_le, Option::<i16>}
    impl_via_inner! {read_i32_le, Option::<i32>}
    impl_via_inner! {read_f32_le, Option::<f32>}

    pub fn read_header_name(&mut self) -> Result<HeaderRef, crate::io::PaddedStringReadError> {
        self.inner.read_c_ascii_str().map(HeaderRef::from)
    }

    impl_via_inner! {read_c_utf8_str, Result<PaddedUtf8Str<1>, crate::io::PaddedStringReadError>}
    impl_via_inner! {read_c_utf8_string_lossy, Result<PaddedUtf8String<1>, crate::io::PaddedStringReadError>}
}
