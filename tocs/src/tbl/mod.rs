mod schema;
pub mod schemas;
mod ser;
pub mod value_conversion;

pub mod fixes;

use indexmap::IndexMap;
use smallvec::SmallVec;

use itertools::Itertools as _;

use schema::SmallBuf;

use crate::io::{read_bytes, PaddedByteString};

pub use schema::{get_nf1_for_all, AtomicAttributeValues, AtomicSchema, AtomicSchemas, AtomicType, AtomicValue, Attribute, Header, Schema, Schemas, Type, Value};
pub use ser::{serialize_tbl_v1, serialize_tbl_v2, SerializeError};
pub use value_conversion::try_atomic_value_from_str;

// tests
#[cfg(all(test, feature = "private-tests"))]
mod test;

// Generic helpers

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

    pub fn ignore(mut self) -> Self {
        self.ignore = true;
        self
    }

    pub fn log(&mut self, warning: W) {
        let Self { ignore, warnings } = self;
        if !*ignore {
            warnings.push(warning);
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.warnings.len()
    }

    pub fn ignores_warnings(&self) -> bool {
        self.ignore
    }

    pub fn clear(&mut self) {
        self.warnings.clear();
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn warnings(&self) -> impl Iterator<Item = &W> {
        self.warnings.iter()
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

// Deserialize, i.e. read tbl

#[derive(Debug)]
pub enum DeserializeErrorDetail {
    Io(std::io::Error),
    Ascii,
    Utf8(std::str::Utf8Error, String),
    SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total: u16, sum: Option<u32> },
    DuplicateHeader(Box<Header>),
    EntryHeaderNotInTblHeader { missing: Box<Header> },
    EntryCountDoesNotMatchHeaderEntry { header: Box<Header>, expected: u32, actual: usize },
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
    pub position: Option<u64>,                            // seek position in the file
    pub context: std::vec::Vec<DeserializeErrorPosition>, // first element is the position of the original cause, the rest is how we got there; may be empty
}

impl detail::ErrorWithContext for DeserializeError {
    type Position = DeserializeErrorPosition;

    fn add_context(&mut self, position: Self::Position) {
        self.context.push(position);
    }
}

impl std::fmt::Display for DeserializeErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DeserializeErrorDetail::Io(e) => write!(f, "{}", e),
            DeserializeErrorDetail::Ascii => write!(f, "invalid ASCII string"),
            DeserializeErrorDetail::Utf8(e, s) => write!(f, "{} for string '{}'", e, s.escape_debug()),
            DeserializeErrorDetail::SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total, sum } => {
                write!(f, "sum of entry numbers ")?;
                if let Some(n) = sum {
                    write!(f, "({})", n)?;
                } else {
                    write!(f, "(exceeds max allowed)",)?;
                }
                write!(f, " differs from expected number ({})", expected_total)
            }
            DeserializeErrorDetail::DuplicateHeader(hdr) => write!(f, "duplicate header '{}'", hdr),
            DeserializeErrorDetail::EntryHeaderNotInTblHeader { missing } => write!(f, "file contains entry with header '{}' which does not appear in the tbl header", missing),
            DeserializeErrorDetail::EntryCountDoesNotMatchHeaderEntry { header, expected, actual } => write!(
                f,
                "number of entries for header '{}' in the file ({}) differs from the count given in the tbl header ({})",
                header, actual, expected
            ),
            DeserializeErrorDetail::EntryDataLengthMismatch { expected, actual } => {
                write!(f, "length of entry data ")?;
                if let Some(len) = actual {
                    write!(f, "({})", len)?;
                } else {
                    write!(f, "(>{})", u16::MAX)?;
                }
                write!(f, " differs from length in table header ({})", expected)
            }
            DeserializeErrorDetail::TrailingData => write!(f, "unexpected trailing data"),
        }
    }
}

impl std::fmt::Display for DeserializeErrorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DeserializeErrorPosition::Start => write!(f, "before parsing"),
            DeserializeErrorPosition::TblHeader => write!(f, "while parsing tbl header"),
            DeserializeErrorPosition::TblHeaderEntry { entry_number } => write!(f, "while parsing tbl header number {}", entry_number + 1),
            DeserializeErrorPosition::Entry { number } => write!(f, "while parsing header of entry number {}", number + 1),
            DeserializeErrorPosition::EntryHeader => write!(f, "while parsing entry header name"),
            DeserializeErrorPosition::EntryData => write!(f, "while parsing entry data"),
            DeserializeErrorPosition::HeaderName => write!(f, "while parsing name of a header"),
            DeserializeErrorPosition::Value(ty) => write!(f, "while parsing a {}", ty.describe()),
            DeserializeErrorPosition::Attribute(name) => write!(f, "while parsing attribute '{}'", name),
            DeserializeErrorPosition::End => write!(f, "after parsing"),
        }
    }
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let Self { detail, position, context } = self;
        write!(f, "{} at ", detail)?;
        if let Some(off) = position {
            write!(f, "offset {} (0x{:x})", off, off)?;
        } else {
            write!(f, "an unknown offset")?;
        }
        if !context.is_empty() {
            write!(f, ". error occured {}", context[0])?;
            for position in &context[1..] {
                write!(f, "\n - {}", position)?;
            }
        }
        Ok(())
    }
}

fn deserialize_c_utf8<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(f: &mut F, warnings: &mut WarningLogger<DeserializeError>) -> Result<smol_str::SmolStr, DeserializeError> {
    let bytes = PaddedByteString::<1>::read(f).map_err(|e| match e {
        crate::io::PaddedStringReadError::Io(e) => DeserializeError {
            detail: DeserializeErrorDetail::Io(e),
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::Value(AtomicType::CUtf8)],
        },
        _ => unreachable!(),
    })?;
    let buf: &[u8] = bytes.as_bytes_excluding_terminator_and_padding();
    match std::str::from_utf8(buf) {
        Ok(s) => Ok(s.into()),
        Err(utf8err) => {
            let s = String::from_utf8_lossy(buf);
            let position = (|| {
                let pos: u64 = f.stream_position().ok()?;
                let pos: usize = pos.try_into().ok()?;
                let pos = pos.checked_sub(buf.len().checked_add(1)?)?.checked_add(utf8err.valid_up_to())?;
                pos.try_into().ok()
            })();
            let e = DeserializeError {
                detail: DeserializeErrorDetail::Utf8(utf8err, s.to_string()),
                position,
                context: vec![DeserializeErrorPosition::Value(AtomicType::CUtf8)],
            };
            if STRICT {
                if LOG {
                    log::error!("{}", e.detail);
                }
                Err(e)
            } else {
                if LOG {
                    log::warn!("{}", e.detail);
                    warnings.log(e);
                }
                Ok(s.into())
            }
        }
    }
}

fn deserialize_header_name<F: std::io::Read + std::io::Seek, const LOG: bool>(f: &mut F) -> Result<Header, DeserializeError> {
    let bytes = PaddedByteString::<1>::read(f).map_err(|e| match e {
        crate::io::PaddedStringReadError::Io(e) => DeserializeError {
            detail: DeserializeErrorDetail::Io(e),
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::HeaderName],
        },
        _ => unreachable!(),
    })?;
    let len = bytes.as_bytes_excluding_terminator_and_padding().len();
    Header::try_from(bytes).map_err(|e| match e {
        crate::io::PaddedStringError::Ascii => DeserializeError {
            detail: DeserializeErrorDetail::Ascii,
            position: f.stream_position().ok().map(|pos| pos - len as u64),
            context: vec![DeserializeErrorPosition::HeaderName],
        },
        _ => unreachable!(),
    })
}

pub fn deserialize_atomic_value<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    t: &AtomicType,
) -> Result<AtomicValue, DeserializeError> {
    let wrap_err = |e: std::io::Error, f: &mut F| DeserializeError {
        detail: DeserializeErrorDetail::Io(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::Value(*t)],
    };
    Ok(match t {
        AtomicType::I8 => AtomicValue::I8(i8::from_le_bytes(read_bytes(f).map_err(|e| wrap_err(e, f))?)),
        AtomicType::I16 => AtomicValue::I16(i16::from_le_bytes(read_bytes(f).map_err(|e| wrap_err(e, f))?)),
        AtomicType::I32 => AtomicValue::I32(i32::from_le_bytes(read_bytes(f).map_err(|e| wrap_err(e, f))?)),
        AtomicType::U8 => AtomicValue::U8(u8::from_le_bytes(read_bytes(f).map_err(|e| wrap_err(e, f))?)),
        AtomicType::U16 => AtomicValue::U16(u16::from_le_bytes(read_bytes(f).map_err(|e| wrap_err(e, f))?)),
        AtomicType::U32 => AtomicValue::U32(u32::from_le_bytes(read_bytes(f).map_err(|e| wrap_err(e, f))?)),
        AtomicType::F32 => AtomicValue::F32(f32::from_le_bytes(read_bytes(f).map_err(|e| wrap_err(e, f))?)),
        AtomicType::CUtf8 => AtomicValue::CUtf8(deserialize_c_utf8::<_, STRICT, LOG>(f, log)?),
        AtomicType::X(n) => {
            let mut buf = SmallBuf::with_capacity(*n);
            for _ in 0..*n {
                buf.push(0);
            }
            f.read_exact(&mut buf).map_err(|e| wrap_err(e, f))?;
            AtomicValue::X(buf)
        }
    })
}

/// deserialize tbl header for trails of cold steel (no list of headers at the start)
fn deserialize_header_v1<F: std::io::Read + std::io::Seek>(f: &mut F) -> Result<u16, DeserializeError> {
    read_bytes(f).map(u16::from_le_bytes).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::Io(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })
}

/// deserialize tbl header for trails of cold steel 2 - 5 (includes list of headers at the start)
fn deserialize_header_v2<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
) -> Result<(u16, IndexMap<Header, u32>), DeserializeError> {
    let warnings_before = log.len();
    let number_of_entries = read_bytes(f).map(u16::from_le_bytes).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::Io(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })?;
    if LOG {
        log::trace!("number of entries according to header: {}", number_of_entries);
    }
    let header_counts = deserialize_header_counts::<_, STRICT, LOG>(f, log)?;
    let sum_of_counts: Option<u32> = header_counts.values().fold(Some(0), |a, b| a?.checked_add(*b));
    if sum_of_counts != Some(number_of_entries.into()) {
        let err = DeserializeError {
            detail: DeserializeErrorDetail::SumOfEntryNumbersDiffersFromTotalEntryNumber {
                expected_total: number_of_entries,
                sum: sum_of_counts,
            },
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::TblHeader],
        };
        if STRICT {
            if LOG {
                log::error!("{}", err.detail);
            }
            return Err(err);
        } else if LOG {
            log::warn!("{}", err.detail);
            log.log(err);
        }
    }
    if LOG {
        log.add_context_for_all_since(warnings_before, DeserializeErrorPosition::TblHeader);
    }
    Ok((number_of_entries, header_counts))
}

fn deserialize_header_counts<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
) -> Result<IndexMap<Header, u32>, DeserializeError> {
    let warnings_before = log.len();
    let number_of_headers = u32::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::Io(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })?);
    if LOG {
        log::trace!("number of headers in the file: {}", number_of_headers);
    }

    let mut header_counts = IndexMap::new();
    for i in 0..number_of_headers {
        let hdr = deserialize_header_name::<_, STRICT>(f)?;
        let count = u32::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
            detail: DeserializeErrorDetail::Io(e),
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::TblHeaderEntry { entry_number: i }],
        })?);
        if LOG {
            log::trace!("header '{}': {} entries", hdr, count);
        }
        let entry = header_counts.entry(hdr);
        match entry {
            indexmap::map::Entry::Occupied(mut e) => {
                let hdr = e.key();
                // TODO: make a macro for this pattern:
                let err = DeserializeError {
                    detail: DeserializeErrorDetail::DuplicateHeader(Box::new(hdr.clone())),
                    position: f.stream_position().ok(),
                    context: vec![DeserializeErrorPosition::TblHeaderEntry { entry_number: i }],
                };
                if STRICT {
                    if LOG {
                        log::error!("duplicate in tbl header: '{}'", hdr);
                    }
                    return Err(err);
                } else if LOG {
                    log::warn!("duplicate in tbl header: '{}'", hdr);
                    log.log(err);
                }
                e.insert(count);
            }
            indexmap::map::Entry::Vacant(e) => {
                e.insert(count);
            }
        }
    }
    if LOG {
        log.add_context_for_all_since(warnings_before, DeserializeErrorPosition::TblHeader);
    }
    Ok(header_counts)
}

pub fn deserialize_entry_data<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    schema: &AtomicSchema,
) -> Result<AtomicAttributeValues, DeserializeError> {
    let mut values = IndexMap::new();
    for (attr, ty) in schema.iter() {
        let warnings_before = log.len();
        let val = deserialize_atomic_value::<_, STRICT, LOG>(f, log, ty).map_err(|e| e.with_context(DeserializeErrorPosition::Attribute(attr.clone())))?;
        if LOG {
            log::trace!("attribute '{}' = {:?}", attr, val);
            log.add_context_for_all_since(warnings_before, DeserializeErrorPosition::Attribute(attr.clone()));
        }
        values.insert(attr.clone(), val);
    }
    Ok(values)
}

fn deserialize_entry<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    header_counts: Option<&IndexMap<Header, u32>>,
    schemas: &AtomicSchemas,
) -> Result<Entry, DeserializeError> {
    // read header name and check that it's ok (if necessary)
    let warnings_before_header = log.len();
    let hdr = deserialize_header_name::<_, STRICT>(f)?;
    if let Some(allowed) = header_counts {
        if !allowed.contains_key(&hdr) {
            let e = DeserializeError {
                detail: DeserializeErrorDetail::EntryHeaderNotInTblHeader { missing: Box::new(hdr.clone()) },
                position: f.stream_position().ok(),
                context: vec![],
            };
            if STRICT {
                if LOG {
                    log::error!("Header '{}' not in tbl header", hdr);
                }
                return Err(e);
            } else if LOG {
                log::warn!("Header '{}' not in tbl header", hdr);
                log.log(e);
            }
        }
    }
    if LOG {
        log::trace!("entry header name: {}", hdr);
        log.add_context_for_all_since(warnings_before_header, DeserializeErrorPosition::EntryHeader);
    }

    let warnings_before_data = log.len();
    // read data length
    let len = u16::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::Io(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::EntryHeader],
    })?);
    if LOG {
        log::trace!("entry data length: {}", len);
    }
    let (hdr, data) = if let Some((hdr, schema)) = schemas.get_key_value(&hdr) {
        if LOG {
            log::debug!("parsing data according to schema");
        }
        let data = deserialize_entry_data::<_, STRICT, LOG>(f, log, schema).map_err(|e| e.with_context(DeserializeErrorPosition::EntryData))?;
        (hdr.clone(), data)
    } else {
        if LOG {
            log::debug!("parsing data with general parser");
        }
        let x = deserialize_atomic_value::<_, STRICT, LOG>(f, log, &AtomicType::X(len.into()))
            .map_err(|e| e.with_context(DeserializeErrorPosition::Attribute("data".into())).with_context(DeserializeErrorPosition::EntryData))?;
        (hdr, [("data".into(), x)].into_iter().collect())
    };
    let actual_len = ser::atomic_attributes_serialization_length(&data);
    if actual_len != Some(len) {
        let e = DeserializeError {
            detail: DeserializeErrorDetail::EntryDataLengthMismatch { expected: len, actual: actual_len },
            position: f.stream_position().ok(),
            context: vec![],
        };
        if STRICT {
            if LOG {
                log::error!("{}", e.detail);
            }
            return Err(e);
        } else if LOG {
            log::warn!("{}", e.detail);
            log.log(e);
        }
    }

    if LOG {
        log.add_context_for_all_since(warnings_before_data, DeserializeErrorPosition::EntryData);
    }
    Ok(Entry { header: hdr, values: data })
}

fn deserialize_all_entries<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    number_of_entries: u16,
    header_counts: Option<&IndexMap<Header, u32>>,
    schemas: &AtomicSchemas,
) -> Result<std::vec::Vec<Entry>, DeserializeError> {
    let mut entries = std::vec::Vec::new();

    // parse number of entries equal to tbl header
    for i in 0..number_of_entries {
        let warnings_before = log.len();
        if LOG {
            log::debug!("entry number {}", i);
        }
        entries.push(deserialize_entry::<_, STRICT, LOG>(f, log, header_counts, schemas).map_err(|e| e.with_context(DeserializeErrorPosition::Entry { number: i }))?);
        log.add_context_for_all_since(warnings_before, DeserializeErrorPosition::Entry { number: i });
    }

    // parse the rest of the files
    let getpos = |f: &mut F| {
        f.stream_position().map_err(|e| DeserializeError {
            detail: DeserializeErrorDetail::Io(e),
            position: None,
            context: vec![DeserializeErrorPosition::End],
        })
    };
    let getlen = |f: &mut F| {
        let pos = f.stream_position()?;
        let len = f.seek(std::io::SeekFrom::End(0))?;
        f.seek(std::io::SeekFrom::Start(pos))?;
        Ok(len)
    };
    let file_len = getlen(f).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::Io(e),
        position: None,
        context: vec![DeserializeErrorPosition::End],
    })?;

    if STRICT {
        // we should be at eof
        let pos = getpos(f)?;
        if pos != file_len {
            if LOG {
                log::error!("error: trailing bytes after offset {pos}");
            }
            return Err(DeserializeError {
                detail: DeserializeErrorDetail::TrailingData,
                position: Some(pos),
                context: vec![DeserializeErrorPosition::End],
            });
        }
    } else {
        let mut i = number_of_entries;
        while getpos(f)? != file_len {
            if LOG {
                log::debug!("parsing another entry");
            }
            let warnings_before = log.len();
            entries.push(deserialize_entry::<_, STRICT, LOG>(f, log, None, schemas)?);
            log.add_context_for_all_since(warnings_before, DeserializeErrorPosition::Entry { number: i });
            i = i.saturating_add(1);
        }
    }

    Ok(entries)
}

pub fn deserialize_tbl_v1<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    schemas: &AtomicSchemas,
) -> Result<std::vec::Vec<Entry>, DeserializeError> {
    if LOG {
        log::debug!("parsing tbl");
    }
    let number_of_entries = deserialize_header_v1(f)?;
    deserialize_all_entries::<_, STRICT, LOG>(f, log, number_of_entries, None, schemas)
}

pub fn deserialize_tbl_v2<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    schemas: &AtomicSchemas,
) -> Result<(IndexMap<Header, u32>, std::vec::Vec<Entry>), DeserializeError> {
    if LOG {
        log::debug!("parsing tbl");
    }
    let (number_of_entries, header_counts) = deserialize_header_v2::<_, STRICT, LOG>(f, log)?;
    let entries = deserialize_all_entries::<_, STRICT, LOG>(f, log, number_of_entries, Some(&header_counts), schemas)?;
    Ok((header_counts, entries))
}

// generic load/fix/store

#[derive(Clone, Copy, Debug)]
enum VersionWithoutSchema {
    V1, // no header counts: CS1
    V2, // with header counts: CS2 onwards
}

#[derive(Clone, Copy, Debug)]
pub enum Version<'a> {
    CS1En,
    CS2En,
    CS3,
    CS4,
    V1(&'a AtomicSchemas), // generic version with custom schema (no header counts / CS1 version)
    V2(&'a AtomicSchemas), // generic version with custom schema (with header counts / CS2 onward)
}

impl std::str::FromStr for Version<'static> {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, &'static str> {
        match s {
            "cs1" => Ok(Version::CS1En),
            "CS1" => Ok(Version::CS1En),
            "cs2" => Ok(Version::CS2En),
            "CS2" => Ok(Version::CS2En),
            "cs3" => Ok(Version::CS3),
            "CS3" => Ok(Version::CS3),
            "cs4" => Ok(Version::CS4),
            "CS4" => Ok(Version::CS4),
            "v1" => Ok(Version::V1(&schemas::EMPTY)),
            "v2" => Ok(Version::V2(&schemas::EMPTY)),
            _ => Err("invalid/unknown tbl version"),
        }
    }
}

impl<'a> Version<'a> {
    fn without_schema(&self) -> VersionWithoutSchema {
        match self {
            Version::CS1En => VersionWithoutSchema::V1,
            Version::CS2En => VersionWithoutSchema::V2,
            Version::CS3 => VersionWithoutSchema::V2,
            Version::CS4 => VersionWithoutSchema::V2,
            Version::V1(_) => VersionWithoutSchema::V1,
            Version::V2(_) => VersionWithoutSchema::V2,
        }
    }

    pub fn schemas(&self) -> &'a AtomicSchemas {
        match self {
            Version::CS1En => &schemas::CS1_ATOMIC,
            Version::CS2En => &schemas::CS2_ATOMIC,
            Version::CS3 => &schemas::CS3_ATOMIC,
            Version::CS4 => &schemas::CS4_ATOMIC,
            Version::V1(schema) => schema,
            Version::V2(schema) => schema,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tbl {
    version: VersionWithoutSchema,
    headers: SmallVec<[Header; 6]>,
    entries: std::vec::Vec<Entry>,
}

impl Tbl {
    pub fn new<I: IntoIterator<Item = Header>, J: IntoIterator<Item = Entry>>(version: Version, headers: I, entries: J) -> Self {
        Self {
            version: version.without_schema(),
            headers: headers.into_iter().collect(),
            entries: entries.into_iter().collect(),
        }
    }

    pub fn headers(&self) -> impl Iterator<Item = &Header> {
        self.headers.iter()
    }

    pub fn get_entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn get_mut_entries(&mut self) -> &mut std::vec::Vec<Entry> {
        &mut self.entries
    }

    pub fn into_entries(self) -> std::vec::Vec<Entry> {
        self.entries
    }
}

// TODO: load/store Tbl from file/data, including fixes (for specific versions)

pub enum ReadTblResult {
    Ok(Tbl),
    Warn { tbl: Tbl, warnings: std::vec::Vec<DeserializeError> },
    Err(DeserializeError),
}

impl ReadTblResult {
    pub fn ignore_warnings(self) -> Result<Tbl, DeserializeError> {
        match self {
            ReadTblResult::Ok(tbl) => Ok(tbl),
            ReadTblResult::Warn { tbl, .. } => Ok(tbl),
            ReadTblResult::Err(e) => Err(e),
        }
    }
}

fn try_read<F: std::io::Read + std::io::Seek, const STRICT: bool, const LOG: bool>(f: &mut F, logger: &mut WarningLogger<DeserializeError>, version: Version) -> Result<Tbl, DeserializeError> {
    match version.without_schema() {
        VersionWithoutSchema::V1 => deserialize_tbl_v1::<_, STRICT, LOG>(f, logger, version.schemas()).map(|entries| Tbl {
            version: version.without_schema(),
            headers: entries.iter().map(|entry| &entry.header).unique().cloned().collect(),
            entries,
        }),
        VersionWithoutSchema::V2 => deserialize_tbl_v2::<_, STRICT, LOG>(f, logger, version.schemas()).map(|(headers_with_counts, entries)| Tbl {
            version: version.without_schema(),
            headers: headers_with_counts.into_iter().map(|(h, _)| h).collect(),
            entries,
        }),
    }
}

fn warnings_are_likely_fine(tbl: &Tbl, schemas: &AtomicSchemas, warnings: &[DeserializeError]) -> bool {
    let mut utf8errors = 0;
    // For errors that appear while parsing an entry, check that the schema for the entry is known.
    // For other errors, check that they are probably minor.
    // While we're at it, also count the number of UTF8 errors.
    for w in warnings.iter() {
        if matches!(w.detail, DeserializeErrorDetail::Utf8(_, _)) {
            utf8errors += 1;
        }
        let mut is_entry_error = false;
        for pos in w.context.iter() {
            match pos {
                DeserializeErrorPosition::Start => {}
                DeserializeErrorPosition::TblHeader => {}
                DeserializeErrorPosition::TblHeaderEntry { .. } => {}
                DeserializeErrorPosition::Entry { number } => {
                    let entries = tbl.get_entries();
                    let number: usize = (*number).try_into().unwrap();
                    let header: Header = entries[number].header.clone();
                    if !schemas.contains_key(&header) {
                        return false;
                    } else {
                        is_entry_error = true;
                    }
                }
                DeserializeErrorPosition::EntryHeader => {}
                DeserializeErrorPosition::EntryData => {}
                DeserializeErrorPosition::HeaderName => {
                    return false;
                }
                DeserializeErrorPosition::Value(_) => {}
                DeserializeErrorPosition::Attribute(_) => {}
                DeserializeErrorPosition::End => {}
            }
        }
        if !is_entry_error {
            match &w.detail {
                DeserializeErrorDetail::Io(_) => unreachable!(),
                DeserializeErrorDetail::Ascii => return false,
                DeserializeErrorDetail::Utf8(_, _) => return false,
                DeserializeErrorDetail::SumOfEntryNumbersDiffersFromTotalEntryNumber { .. } => {} // it's probably fine
                DeserializeErrorDetail::DuplicateHeader(_) => {}
                DeserializeErrorDetail::EntryHeaderNotInTblHeader { .. } => {}
                DeserializeErrorDetail::EntryCountDoesNotMatchHeaderEntry { header, .. } => {
                    if !schemas.contains_key(header.as_ref()) {
                        return false; // otherwise it's probably fine
                    }
                }
                DeserializeErrorDetail::EntryDataLengthMismatch { .. } => {
                    return false;
                }
                DeserializeErrorDetail::TrailingData => {
                    return false;
                }
            }
        }
    }

    // if there are too many utf8 errors, there's probably an encoding mismatch or something
    if utf8errors > (tbl.get_entries().len() / 10).clamp(1, 10) {
        return false;
    }

    // seems fine
    true
}

pub fn read<F: std::io::Read + std::io::Seek>(f: &mut F, version: Version) -> ReadTblResult {
    log::debug!("Generic tbl read");
    let mut logger = WarningLogger::new();
    let first_error = match try_read::<_, /*STRICT =*/ true, /*LOG=*/ false>(f, &mut logger, version) {
        Ok(tbl) => {
            return ReadTblResult::Ok(tbl);
        }
        Err(e) => e,
    };

    log::debug!("STRICT tbl read got error: {}. Trying to read again (non-strict).", first_error);
    if let Err(e) = f.rewind() {
        log::warn!("Tried to reread the table non-strict but rewinding failed");
        return ReadTblResult::Err(DeserializeError {
            detail: DeserializeErrorDetail::Io(e),
            position: f.stream_position().ok(),
            context: vec![], // TODO: new ErrorPosition just for this?
        });
    }
    logger.clear();
    log::debug!("Rereading the tbl non-strictly");
    if let Ok(tbl) = try_read::<_, /*STRICT =*/ false, /*LOG=*/ true>(f, &mut logger, version) {
        let warnings = logger.into_warnings();
        // do all entries in the file for which there are warnings have known schemas?
        if warnings_are_likely_fine(&tbl, version.schemas(), &warnings) {
            log::debug!("Reread the tbl non-strictly, seems fine.");
            return ReadTblResult::Warn { tbl, warnings };
        }
    }

    ReadTblResult::Err(first_error)
}

pub fn write<F: std::io::Write>(f: &mut F, tbl: &Tbl) -> Result<usize, SerializeError> {
    let written = match tbl.version {
        VersionWithoutSchema::V1 => serialize_tbl_v1(f, &tbl.entries)?,
        VersionWithoutSchema::V2 => serialize_tbl_v2(f, &tbl.headers, &tbl.entries)?,
    };
    f.flush().map_err(SerializeError::IoError)?;
    Ok(written)
}

pub fn read_file<P: AsRef<std::path::Path>>(path: P, version: Version) -> ReadTblResult {
    match std::fs::File::open(path.as_ref()) {
        Ok(f) => read(&mut std::io::BufReader::new(f), version),
        Err(e) => ReadTblResult::Err(DeserializeError {
            detail: DeserializeErrorDetail::Io(e),
            position: None,
            context: vec![DeserializeErrorPosition::Start],
        }),
    }
}

pub fn write_file<P: AsRef<std::path::Path>>(path: P, tbl: &Tbl) -> Result<usize, SerializeError> {
    std::fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path.as_ref())
        .map_err(SerializeError::IoError)
        .and_then(|f| write(&mut std::io::BufWriter::new(f), tbl))
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub header: Header,
    pub values: AtomicAttributeValues,
}
