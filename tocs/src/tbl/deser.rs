use crate::io::{read_bytes, PaddedByteString};

use super::{AtomicType, Attribute, Header};

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
