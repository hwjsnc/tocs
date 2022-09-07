use hex::FromHex as _;
use std::io::Write as _;
use std::str::FromStr as _;

use indexmap::IndexMap;
use smallvec::SmallVec;
use smol_str::SmolStr;

#[cfg(feature = "rusqlite")]
use rusqlite::types::FromSql as _;

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
}

// Schema definition types

// Types as they appear in the schema definition
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    I8,
    I16,
    I32,
    //I64,
    U8,
    U16,
    U32,
    //U64,
    F32,
    //F64,
    CUtf8, // null terminated UTF-8 string
    //C_SJIS, // null terminated Shift-JIS string
    X(usize),
    Repeat(usize, Box<Type>),
    Ref(SmolStr),
}

// Values for parsing/serializing
#[derive(Clone, Debug)]
pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    U8(u8),
    U16(u16),
    U32(u32),
    CUtf8(String),
    //C_SJIS(SJISString),
    X(std::vec::Vec<u8>),
    Repeat(std::vec::Vec<Value>),
    Ref(Box<Value>),
}

// Atomic types; after unrolling repetitions and dereferencing references -- for converting to/from CSV/SQL
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AtomicType {
    I8,
    I16,
    I32,
    U8,
    U16,
    U32,
    F32,
    CUtf8,
    X(usize),
}

pub type SmallBuf = SmallVec<[u8; 16]>;

// Atomic values - values corresponding to the atomic types
#[derive(Clone, Debug)]
pub enum AtomicValue {
    I8(i8),
    I16(i16),
    I32(i32),
    U8(u8),
    U16(u16),
    U32(u32),
    F32(f32),
    CUtf8(SmolStr),
    X(SmallBuf),
}

// Type -> Atomic Types (unrolling sequences, dereferencing common structures such as effects)

#[derive(Clone, Debug)]
pub enum NF1ErrorDetail {
    NoSuchSchema(SmolStr),
    Duplicate(SmolStr),
}

#[derive(Clone, Debug)]
pub enum NF1ErrorPosition {
    SchemaLookup,
    Deref(SmolStr),
}

#[derive(Clone, Debug)]
pub struct NF1Error {
    detail: NF1ErrorDetail,
    context: std::vec::Vec<NF1ErrorPosition>, // first element is the position of the original cause, the rest is how we got there; may be empty
}

impl NF1Error {
    pub fn add_context(mut self, position: NF1ErrorPosition) -> Self {
        self.context.push(position);
        self
    }
}

impl std::fmt::Display for NF1ErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            NF1ErrorDetail::NoSuchSchema(s) => write!(f, "no schema '{}' found", s),
            NF1ErrorDetail::Duplicate(s) => write!(f, "duplicate definition of schema '{}'", s),
        }
    }
}

impl std::fmt::Display for NF1ErrorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            NF1ErrorPosition::SchemaLookup => write!(f, "during schema lookup"),
            NF1ErrorPosition::Deref(s) => write!(f, "while dereferencing a subschema of '{}'", s),
        }
    }
}

impl std::fmt::Display for NF1Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let Self { detail, context } = self;
        write!(f, "schema normalization error: {}", detail)?;
        if !context.is_empty() {
            write!(f, ". error occured {}", context[0])?;
            for pos in &context[1..] {
                write!(f, "\n - {}", pos)?;
            }
        }
        Ok(())
    }
}

impl Type {
    fn try_to_atomic_type(&self) -> Option<AtomicType> {
        match self {
            Type::I8 => Some(AtomicType::I8),
            Type::I16 => Some(AtomicType::I16),
            Type::I32 => Some(AtomicType::I32),
            Type::U8 => Some(AtomicType::U8),
            Type::U16 => Some(AtomicType::U16),
            Type::U32 => Some(AtomicType::U32),
            Type::F32 => Some(AtomicType::F32),
            Type::CUtf8 => Some(AtomicType::CUtf8),
            Type::X(n) => Some(AtomicType::X(*n)),
            Type::Repeat(_, _) => None,
            Type::Ref(_) => None,
        }
    }
}

// compute the first normal form, i.e. unroll sequences (except data of a given length) and dereference references.
pub fn get_nf1(schema_name: &SmolStr, schemas: &IndexMap<SmolStr, IndexMap<SmolStr, Type>>) -> Result<IndexMap<SmolStr, AtomicType>, NF1Error> {
    // TODO: check duplicates
    let schema = schemas.get(schema_name).ok_or_else(|| NF1Error {
        detail: NF1ErrorDetail::NoSuchSchema(schema_name.clone()),
        context: vec![NF1ErrorPosition::SchemaLookup],
    })?;
    let mut nf1 = IndexMap::new();
    for (name, ty) in schema.iter() {
        if let Some(t) = ty.try_to_atomic_type() {
            nf1.insert(name.clone(), t);
            continue;
        }
        match ty {
            Type::Repeat(n, ty) => {
                if let Some(t) = ty.try_to_atomic_type() {
                    for i in 1..=*n {
                        nf1.insert(SmolStr::new(format!("{} {}", name, i)), t);
                    }
                } else if let Type::Ref(s) = ty.as_ref() {
                    let sub_nf1 = get_nf1(s, schemas).map_err(|e| e.add_context(NF1ErrorPosition::Deref(schema_name.clone())))?;
                    for i in 1..=*n {
                        for (n, t) in sub_nf1.iter() {
                            nf1.insert(SmolStr::new(format!("{}[{}] {}", name, i, n)), *t);
                        }
                    }
                } else {
                    panic!("unsupported type for repeat in normal form: {:?}", ty);
                }
            }
            Type::Ref(s) => {
                let sub_nf1 = get_nf1(s, schemas).map_err(|e| e.add_context(NF1ErrorPosition::Deref(schema_name.clone())))?;
                for (n, t) in sub_nf1.into_iter() {
                    nf1.insert(SmolStr::new(format!("{} {}", name, n)), t);
                }
            }
            _ => unreachable!("all other types should be primitive and caught by the 'if let' above"),
        }
    }
    Ok(nf1)
}

pub fn get_nf1_for_all(
    schemas: &IndexMap<SmolStr, IndexMap<SmolStr, Type>>,
    common: &IndexMap<SmolStr, IndexMap<SmolStr, Type>>,
) -> Result<IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>>, NF1Error> {
    let mut joint = schemas.clone();
    joint.extend(common.clone().into_iter());
    let joint = joint;
    let mut res = IndexMap::new();
    for name in schemas.keys() {
        res.insert(name.clone(), get_nf1(name, &joint)?);
    }
    Ok(res)
}

// Conversion to and from strings

pub enum AtomicValueFromStringError {
    Generic,
    LengthMismatch { expected: usize, actual: usize },
}

pub fn try_atomic_value_from_str(t: &AtomicType, s: &str) -> Result<AtomicValue, AtomicValueFromStringError> {
    match t {
        AtomicType::I8 => Ok(AtomicValue::I8(i8::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::I16 => Ok(AtomicValue::I16(i16::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::I32 => Ok(AtomicValue::I32(i32::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::U8 => Ok(AtomicValue::U8(u8::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::U16 => Ok(AtomicValue::U16(u16::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::U32 => Ok(AtomicValue::U32(u32::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::F32 => Ok(AtomicValue::F32(f32::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::CUtf8 => Ok(AtomicValue::CUtf8(SmolStr::from(s))),
        AtomicType::X(n) => {
            let data = std::vec::Vec::<u8>::from_hex(s).map_err(|_| AtomicValueFromStringError::Generic)?; // TODO: SmallVec<_> from hex
            if data.len() != *n {
                Err(AtomicValueFromStringError::LengthMismatch { expected: *n, actual: data.len() })
            } else {
                Ok(AtomicValue::X(data.into()))
            }
        }
    }
}

pub fn atomic_value_to_string(v: &AtomicValue) -> impl AsRef<str> {
    match v {
        AtomicValue::I8(x) => x.to_string(),
        AtomicValue::I16(x) => x.to_string(),
        AtomicValue::I32(x) => x.to_string(),
        AtomicValue::U8(x) => x.to_string(),
        AtomicValue::U16(x) => x.to_string(),
        AtomicValue::U32(x) => x.to_string(),
        AtomicValue::F32(x) => x.to_string(),
        AtomicValue::CUtf8(x) => x.to_string(),
        AtomicValue::X(x) => hex::encode(x),
    }
}

pub fn atomic_type(v: &AtomicValue) -> AtomicType {
    match v {
        AtomicValue::I8(_) => AtomicType::I8,
        AtomicValue::I16(_) => AtomicType::I16,
        AtomicValue::I32(_) => AtomicType::I32,
        AtomicValue::U8(_) => AtomicType::U8,
        AtomicValue::U16(_) => AtomicType::U16,
        AtomicValue::U32(_) => AtomicType::U32,
        AtomicValue::F32(_) => AtomicType::F32,
        AtomicValue::CUtf8(_) => AtomicType::CUtf8,
        AtomicValue::X(data) => AtomicType::X(data.len()),
    }
}

pub fn atomic_type_describe(t: &AtomicType) -> &'static str {
    match t {
        AtomicType::I8 => "i8 (-128 to 127 inclusive)",
        AtomicType::I16 => "i16 (-32768 to 32767 inclusive)",
        AtomicType::I32 => "i32 (-2147483648 to 2147483647 inclusive)",
        AtomicType::U8 => "u8 (0 to 255 inclusive)",
        AtomicType::U16 => "u16 (0 to 65535 inclusive)",
        AtomicType::U32 => "u32 (0 to 4294967295 inclusive)",
        AtomicType::F32 => "f32 (single-precision floating point)",
        AtomicType::CUtf8 => "UTF-8 string",
        AtomicType::X(_) => "hex-encoded data",
    }
}

// conversion to and from sqlite

#[cfg(feature = "rusqlite")]
impl rusqlite::ToSql for AtomicValue {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            AtomicValue::I8(x) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer((*x).into()))),
            AtomicValue::I16(x) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer((*x).into()))),
            AtomicValue::I32(x) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer((*x).into()))),
            AtomicValue::U8(x) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer((*x).into()))),
            AtomicValue::U16(x) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer((*x).into()))),
            AtomicValue::U32(x) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer((*x).into()))),
            AtomicValue::F32(x) => {
                // We want "nice" numbers when editing the f32 as f64, so we go through String which implements a pretty-print algorithm with appropriate rounding.
                // This conversion is no less accurate than "as" conversion, only slower, but this shouldn't matter.
                let s = x.to_string();
                let y = str::parse::<f64>(&s).expect("f32 string should parse as f64");
                Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Real(y)))
            }
            AtomicValue::CUtf8(s) => Ok(rusqlite::types::ToSqlOutput::Borrowed(rusqlite::types::ValueRef::Text(s.as_ref()))),
            AtomicValue::X(data) => Ok(rusqlite::types::ToSqlOutput::Borrowed(rusqlite::types::ValueRef::Blob(&data))),
        }
    }
}

#[cfg(feature = "rusqlite")]
/// attempt to convert a sqlite type to a given atomic type.
///
/// The blob size is intentionally ignored in the type (i.e. if the blob length does not match the length of the input value, this issue is ignored.)
///
/// Integer/float conversion to blob is currently unspecified.
impl AtomicValue {
    pub fn from_sql_value_ref(ty: AtomicType, val: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match ty {
            AtomicType::I8 => i8::column_result(val).map(AtomicValue::I8),
            AtomicType::I16 => i16::column_result(val).map(AtomicValue::I16),
            AtomicType::I32 => i32::column_result(val).map(AtomicValue::I32),
            AtomicType::U8 => u8::column_result(val).map(AtomicValue::U8),
            AtomicType::U16 => u16::column_result(val).map(AtomicValue::U16),
            AtomicType::U32 => u32::column_result(val).map(AtomicValue::U32),
            AtomicType::F32 => f32::column_result(val).map(AtomicValue::F32),
            AtomicType::CUtf8 => val.as_str().map(SmolStr::from).map(AtomicValue::CUtf8),
            AtomicType::X(_) => val.as_blob().map(SmallBuf::from).map(AtomicValue::X),
        }
    }
}

#[cfg(feature = "rusqlite")]
impl AtomicType {
    pub fn sqlite_type(&self) -> &'static str {
        match self {
            AtomicType::I8 | AtomicType::I16 | AtomicType::I32 | AtomicType::U8 | AtomicType::U16 | AtomicType::U32 => "int",
            AtomicType::F32 => "real",
            AtomicType::CUtf8 => "text",
            AtomicType::X(_) => "blob",
        }
    }
}

// Serialize, i.e. generate tbl data

pub fn atomic_schema_serialization_length(schema: &IndexMap<SmolStr, AtomicValue>) -> Result<u16, ()> {
    schema.iter().map(|(_, v)| atomic_value_serialization_length(v)).fold(Ok(0), |a, b| {
        let x = a?;
        let y = b?;
        x.checked_add(y).ok_or(())
    })
}

pub fn atomic_value_serialization_length(v: &AtomicValue) -> Result<u16, ()> {
    match v {
        AtomicValue::I8(_) => Ok(1),
        AtomicValue::I16(_) => Ok(2),
        AtomicValue::I32(_) => Ok(4),
        AtomicValue::U8(_) => Ok(1),
        AtomicValue::U16(_) => Ok(2),
        AtomicValue::U32(_) => Ok(4),
        AtomicValue::F32(_) => Ok(4),
        AtomicValue::CUtf8(s) => (s.len() + 1).try_into().map_err(|_| ()),
        AtomicValue::X(data) => data.len().try_into().map_err(|_| ()),
    }
}

#[derive(Debug)]
pub enum SerializeError {
    IOError(std::io::Error),
    TooManyHeaders,
    TooManyEntries,
    EntryDataTooLong,
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SerializeError::IOError(e) => write!(f, "{}", e),
            SerializeError::TooManyHeaders => write!(f, "too many headers in tbl (max 4294967295)"),
            SerializeError::TooManyEntries => write!(f, "too many entries in tbl (max 65535)"),
            SerializeError::EntryDataTooLong => write!(f, "entry data too long (max 65535bytes))"),
        }
    }
}

pub fn serialize_c_utf8<F: std::io::Write>(f: &mut F, s: &str) -> Result<usize, std::io::Error> {
    let mut written = f.write(s.as_bytes())?;
    assert!(!s.as_bytes().iter().any(|x| *x == 0));
    written += f.write(&[0])?;
    Ok(written)
}

pub fn serialize_atomic_value<F: std::io::Write>(f: &mut F, v: &AtomicValue) -> Result<usize, std::io::Error> {
    match v {
        AtomicValue::I8(x) => f.write(&x.to_le_bytes()),
        AtomicValue::I16(x) => f.write(&x.to_le_bytes()),
        AtomicValue::I32(x) => f.write(&x.to_le_bytes()),
        AtomicValue::U8(x) => f.write(&x.to_le_bytes()),
        AtomicValue::U16(x) => f.write(&x.to_le_bytes()),
        AtomicValue::U32(x) => f.write(&x.to_le_bytes()),
        AtomicValue::F32(x) => f.write(&x.to_le_bytes()),
        AtomicValue::CUtf8(s) => serialize_c_utf8(f, s.as_ref()),
        AtomicValue::X(data) => f.write(data),
    }
}

pub fn nf1_schema_serialize<F: std::io::Write>(f: &mut F, header: &str, attributes: &IndexMap<SmolStr, AtomicValue>) -> Result<usize, SerializeError> {
    let mut written = 0;
    written += serialize_c_utf8(f, header).map_err(SerializeError::IOError)?;
    let length = atomic_schema_serialization_length(attributes).map_err(|()| SerializeError::EntryDataTooLong)?;
    written += f.write(&length.to_le_bytes()).map_err(SerializeError::IOError)?;
    for (_, v) in attributes.iter() {
        written += serialize_atomic_value(f, v).map_err(SerializeError::IOError)?;
    }
    Ok(written)
}

pub fn serialize_tbl_v1<F: std::io::Write>(f: &mut F, entries: &[(SmolStr, IndexMap<SmolStr, AtomicValue>)]) -> Result<usize, SerializeError> {
    let mut written = 0;
    let number_of_entries: Result<u16, _> = entries.len().try_into();
    match number_of_entries {
        Ok(n) => {
            written += f.write(&n.to_le_bytes()).map_err(SerializeError::IOError)?;
        }
        Err(_) => {
            log::error!("too many entries in file ({})", entries.len());
            return Err(SerializeError::TooManyEntries);
        }
    }
    for (header, attrs) in entries {
        written += nf1_schema_serialize(f, header, attrs)?;
    }
    Ok(written)
}

pub fn serialize_tbl_v2<F: std::io::Write>(f: &mut F, headers: &[SmolStr], entries: &[(SmolStr, IndexMap<SmolStr, AtomicValue>)]) -> Result<usize, SerializeError> {
    let mut written = 0;
    let number_of_entries: Result<u16, _> = entries.len().try_into();
    match number_of_entries {
        Ok(n) => {
            written += f.write(&n.to_le_bytes()).map_err(SerializeError::IOError)?;
        }
        Err(_) => {
            log::error!("too many entries in file ({})", entries.len());
            return Err(SerializeError::TooManyEntries);
        }
    }
    let number_of_headers: u32 = headers.len().try_into().map_err(|_| SerializeError::TooManyHeaders)?;
    written += f.write(&number_of_headers.to_le_bytes()).map_err(SerializeError::IOError)?;
    let header_counts = headers.iter().map(|hdr| {
        let n: u32 = entries.iter().filter(|(s, _)| s == hdr).count().try_into().expect("total number of entries fits in a u16");
        (hdr, n)
    });
    for (h, n) in header_counts {
        written += serialize_c_utf8(f, h.as_ref()).map_err(SerializeError::IOError)?;
        written += f.write(&n.to_le_bytes()).map_err(SerializeError::IOError)?;
    }
    for (header, attrs) in entries {
        written += nf1_schema_serialize(f, header, attrs)?;
    }
    Ok(written)
}

pub fn save_file_v1<P: AsRef<std::path::Path>>(path: P, entries: &[(SmolStr, IndexMap<SmolStr, AtomicValue>)]) -> Result<usize, SerializeError> {
    let mut f = std::io::BufWriter::new(std::fs::File::options().create(true).write(true).open(path).map_err(SerializeError::IOError)?);
    let written = serialize_tbl_v1(&mut f, entries)?;
    f.flush().map_err(SerializeError::IOError)?;
    Ok(written)
}

pub fn save_file_v2<P: AsRef<std::path::Path>>(path: P, headers: &[SmolStr], entries: &[(SmolStr, IndexMap<SmolStr, AtomicValue>)]) -> Result<usize, SerializeError> {
    let mut f = std::io::BufWriter::new(std::fs::File::options().create(true).write(true).open(path).map_err(SerializeError::IOError)?);
    let written = serialize_tbl_v2(&mut f, headers, entries)?;
    f.flush().map_err(SerializeError::IOError)?;
    Ok(written)
}

// Deserialize, i.e. read tbl

#[derive(Debug)]
pub enum DeserializeErrorDetail {
    IO(std::io::Error),
    UTF8(std::str::Utf8Error, String),
    SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total: u16, sum: Option<u32> },
    DuplicateHeader(SmolStr),
    EntryHeaderNotInTblHeader { missing: SmolStr },
    EntryCountDoesNotMatchHeaderEntry { header: SmolStr, expected: u32, actual: usize },
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
    Value(AtomicType),
    Attribute(SmolStr),
    End,
}

#[derive(Debug)]
pub struct DeserializeError {
    detail: DeserializeErrorDetail,
    position: Option<u64>,                            // seek position in the file
    context: std::vec::Vec<DeserializeErrorPosition>, // first element is the position of the original cause, the rest is how we got there; may be empty
}

impl std::fmt::Display for DeserializeErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DeserializeErrorDetail::IO(e) => write!(f, "{}", e),
            DeserializeErrorDetail::UTF8(e, s) => write!(f, "{} for string '{}'", e, s.escape_debug()),
            DeserializeErrorDetail::SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total, sum } => {
                write!(f, "sum of entry numbers ")?;
                if let Some(n) = sum {
                    write!(f, "({})", n)?;
                } else {
                    write!(f, "(exceeds max allowed)",)?;
                }
                write!(f, " exceeds expected number ({})", expected_total)
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
            DeserializeErrorPosition::Value(ty) => write!(f, "while parsing a {}", atomic_type_describe(ty)),
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

fn read_byte<F: std::io::Read>(f: &mut F) -> Result<u8, std::io::Error> {
    let a: [u8; 1] = read_bytes(f)?;
    Ok(a[0])
}

fn read_bytes<const N: usize, F: std::io::Read>(f: &mut F) -> Result<[u8; N], std::io::Error> {
    let mut buf: [u8; N] = [0; N];
    f.read_exact(&mut buf)?;
    //Ok(buf.try_into().expect("length should match"))
    Ok(buf)
}

fn deserialize_c_utf8<F: std::io::Read + std::io::Seek>(f: &mut F, warnings: &mut WarningLogger<DeserializeError>, strict: bool) -> Result<SmolStr, DeserializeError> {
    let mut buf: smallvec::SmallVec<[u8; 256]> = smallvec::SmallVec::new();
    loop {
        let x = read_byte(f).map_err(|e| DeserializeError {
            detail: DeserializeErrorDetail::IO(e),
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::Value(AtomicType::CUtf8)],
        })?;
        if x != 0 {
            buf.push(x);
        } else {
            break;
        }
    }
    match std::str::from_utf8(buf.as_ref()) {
        Ok(s) => Ok(SmolStr::from(s)),
        Err(utf8err) => {
            let s = String::from_utf8_lossy(buf.as_ref());
            let position = (|| {
                let pos: u64 = f.stream_position().ok()?;
                let pos: usize = pos.try_into().ok()?;
                let pos = pos.checked_sub(buf.len().checked_add(1)?)?.checked_add(utf8err.valid_up_to())?;
                pos.try_into().ok()
            })();
            let e = DeserializeError {
                detail: DeserializeErrorDetail::UTF8(utf8err, s.to_string()),
                position,
                context: vec![DeserializeErrorPosition::Value(AtomicType::CUtf8)],
            };
            if strict {
                log::error!("{e}");
                Err(e)
            } else {
                log::warn!("{e}");
                warnings.log(e);
                Ok(SmolStr::from(s))
            }
        }
    }
}

pub fn deserialize_atomic_value<F: std::io::Read + std::io::Seek>(f: &mut F, log: &mut WarningLogger<DeserializeError>, strict: bool, t: &AtomicType) -> Result<AtomicValue, DeserializeError> {
    let wrap_err = |e: std::io::Error, f: &mut F| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
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
        AtomicType::CUtf8 => AtomicValue::CUtf8(deserialize_c_utf8(f, log, strict)?),
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
        detail: DeserializeErrorDetail::IO(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })
}

/// deserialize tbl header for trails of cold steel 2 - 5 (includes list of headers at the start)
fn deserialize_header_v2<F: std::io::Read + std::io::Seek>(f: &mut F, log: &mut WarningLogger<DeserializeError>, strict: bool) -> Result<(u16, IndexMap<SmolStr, u32>), DeserializeError> {
    let number_of_entries = read_bytes(f).map(u16::from_le_bytes).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })?;
    log::trace!("number of entries according to header: {}", number_of_entries);
    let header_counts = deserialize_header_counts(f, log, strict)?;
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
        if strict {
            log::error!("entry count ({}) does not match sum of counts per header ({:?})", number_of_entries, sum_of_counts);
            return Err(err);
        } else {
            log::warn!("entry count ({}) does not match sum of counts per header ({:?})", number_of_entries, sum_of_counts);
            log.log(err);
        }
    }
    Ok((number_of_entries, header_counts))
}

fn deserialize_header_counts<F: std::io::Read + std::io::Seek>(f: &mut F, log: &mut WarningLogger<DeserializeError>, strict: bool) -> Result<IndexMap<SmolStr, u32>, DeserializeError> {
    let number_of_headers = u32::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })?);
    log::trace!("number of headers in the file: {}", number_of_headers);

    let mut header_counts = IndexMap::new();
    for i in 0..number_of_headers {
        let hdr = deserialize_c_utf8(f, log, strict).map_err(|mut e| {
            e.context.push(DeserializeErrorPosition::TblHeaderEntry { entry_number: i });
            e
        })?;
        let count = u32::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
            detail: DeserializeErrorDetail::IO(e),
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::TblHeaderEntry { entry_number: i }],
        })?);
        log::trace!("header '{}': {} entries", hdr, count);
        let entry = header_counts.entry(hdr);
        match entry {
            indexmap::map::Entry::Occupied(mut e) => {
                let hdr = e.key();
                // TODO: make a macro for this pattern:
                let err = DeserializeError {
                    detail: DeserializeErrorDetail::DuplicateHeader(hdr.clone()),
                    position: f.stream_position().ok(),
                    context: vec![DeserializeErrorPosition::TblHeaderEntry { entry_number: i }],
                };
                if strict {
                    log::error!("duplicate in tbl header: '{}'", hdr);
                    return Err(err);
                } else {
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
    Ok(header_counts)
}

pub fn deserialize_entry_data<F: std::io::Read + std::io::Seek>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schema: &IndexMap<SmolStr, AtomicType>,
) -> Result<IndexMap<SmolStr, AtomicValue>, DeserializeError> {
    // TODO: use smol_str for index?
    let mut values = IndexMap::new();
    for (attr, ty) in schema.iter() {
        let val = deserialize_atomic_value(f, log, strict, ty).map_err(|mut e| {
            e.context.push(DeserializeErrorPosition::Attribute(attr.clone()));
            e
        })?;
        log::trace!("attribute '{}' = {:?}", attr, val);
        values.insert(attr.clone(), val);
    }
    Ok(values)
}

fn deserialize_entry<F: std::io::Read + std::io::Seek>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    header_counts: Option<&IndexMap<SmolStr, u32>>,
    schemas: &IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>>,
) -> Result<(SmolStr, IndexMap<SmolStr, AtomicValue>), DeserializeError> {
    // read header name and check that it's ok (if necessary)
    let hdr = deserialize_c_utf8(f, log, strict).map_err(|mut e| {
        e.context.push(DeserializeErrorPosition::EntryHeader);
        e
    })?;
    if let Some(allowed) = header_counts {
        if !allowed.contains_key(&hdr as &str) {
            let e = DeserializeError {
                detail: DeserializeErrorDetail::EntryHeaderNotInTblHeader { missing: hdr.clone() },
                position: f.stream_position().ok(),
                context: vec![],
            };
            if strict {
                log::error!("Header '{}' not in tbl header", hdr);
                return Err(e);
            } else {
                log::warn!("Header '{}' not in tbl header", hdr);
                log.log(e);
            }
        }
    }
    log::trace!("entry header name: {}", hdr);
    // read data length
    let len = u16::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: f.stream_position().ok(),
        context: vec![],
    })?);
    log::trace!("entry data length: {}", len);
    let (hdr, data) = if let Some((hdr, schema)) = schemas.get_key_value(&hdr as &str) {
        log::debug!("parsing data according to schema");
        let data = deserialize_entry_data(f, log, strict, schema).map_err(|mut e| {
            e.context.push(DeserializeErrorPosition::EntryData);
            e
        })?;
        (hdr.clone(), data)
    } else {
        log::debug!("parsing data with general parser");
        let x = deserialize_atomic_value(f, log, strict, &AtomicType::X(len.into())).map_err(|mut e| {
            e.context.push(DeserializeErrorPosition::Attribute(SmolStr::new("data")));
            e.context.push(DeserializeErrorPosition::EntryData);
            e
        })?;
        (hdr, [(SmolStr::new("data"), x)].into_iter().collect())
    };
    let actual_len = atomic_schema_serialization_length(&data).ok();
    if actual_len != Some(len) {
        let e = DeserializeError {
            detail: DeserializeErrorDetail::EntryDataLengthMismatch { expected: len, actual: actual_len },
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::EntryData],
        };
        if strict {
            log::error!("{e}");
            return Err(e);
        } else {
            log::warn!("{e}");
            log.log(e);
        }
    }
    Ok((hdr, data))
}

fn deserialize_all_entries<F: std::io::Read + std::io::Seek>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    number_of_entries: u16,
    strict: bool,
    header_counts: Option<&IndexMap<SmolStr, u32>>,
    schemas: &IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>>,
) -> Result<std::vec::Vec<(SmolStr, IndexMap<SmolStr, AtomicValue>)>, DeserializeError> {
    let mut entries = std::vec::Vec::new();

    // parse number of entries equal to tbl header
    for i in 0..number_of_entries {
        log::debug!("entry number {}", i);
        entries.push(deserialize_entry(f, log, strict, header_counts, schemas).map_err(|mut e| {
            e.context.push(DeserializeErrorPosition::Entry { number: i });
            e
        })?);
    }

    // parse the rest of the files
    let getpos = |f: &mut F| {
        f.stream_position().map_err(|e| DeserializeError {
            detail: DeserializeErrorDetail::IO(e),
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
        detail: DeserializeErrorDetail::IO(e),
        position: None,
        context: vec![DeserializeErrorPosition::End],
    })?;

    if strict {
        // we should be at eof
        let pos = getpos(f)?;
        if pos != file_len {
            log::error!("error: trailing bytes after offset {pos}");
            return Err(DeserializeError {
                detail: DeserializeErrorDetail::TrailingData,
                position: Some(pos),
                context: vec![DeserializeErrorPosition::End],
            });
        }
    } else {
        while getpos(f)? != file_len {
            log::debug!("parsing another entry");
            entries.push(deserialize_entry(f, log, strict, None, schemas)?);
        }
    }

    Ok(entries)
}

pub fn deserialize_tbl_v1<F: std::io::Read + std::io::Seek>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schemas: &IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>>,
) -> Result<std::vec::Vec<(SmolStr, IndexMap<SmolStr, AtomicValue>)>, DeserializeError> {
    log::debug!("parsing tbl");
    let number_of_entries = deserialize_header_v1(f)?;
    deserialize_all_entries(f, log, number_of_entries, strict, None, schemas)
}

pub fn deserialize_tbl_v2<F: std::io::Read + std::io::Seek>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schemas: &IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>>,
) -> Result<(IndexMap<SmolStr, u32>, std::vec::Vec<(SmolStr, IndexMap<SmolStr, AtomicValue>)>), DeserializeError> {
    log::debug!("parsing tbl");
    let (number_of_entries, header_counts) = deserialize_header_v2(f, log, strict)?;
    let entries = deserialize_all_entries(f, log, number_of_entries, strict, Some(&header_counts), schemas)?;
    Ok((header_counts, entries))
}

pub fn load_file_v1<P: AsRef<std::path::Path>>(
    path: P,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schemas: &IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>>,
) -> Result<std::vec::Vec<(SmolStr, IndexMap<SmolStr, AtomicValue>)>, DeserializeError> {
    let mut f = std::io::BufReader::new(std::fs::File::open(path).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: None,
        context: vec![DeserializeErrorPosition::Start],
    })?);
    deserialize_tbl_v1(&mut f, log, strict, schemas)
}

pub fn load_file_v2<P: AsRef<std::path::Path>>(
    path: P,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schemas: &IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>>,
) -> Result<(IndexMap<SmolStr, u32>, std::vec::Vec<(SmolStr, IndexMap<SmolStr, AtomicValue>)>), DeserializeError> {
    let mut f = std::fs::File::open(path).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: None,
        context: vec![DeserializeErrorPosition::Start],
    })?;
    deserialize_tbl_v2(&mut f, log, strict, schemas)
}

// schemas

lazy_static::lazy_static! {
    pub static ref SCHEMAS_CS1 : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = [
        (SmolStr::new("ActiveVoiceTableData"), [
            ("unknown_short_1", Type::U16),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_data_1", Type::X(4)),
            ("unknown_string_2", Type::CUtf8),
            ("unknown_float_1", Type::F32),
            ("unknown_float_2", Type::F32),
            ("unknown_data", Type::X(12)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("AttachTableData"), [
            ("character", Type::U16),
            ("unknown_data", Type::X(20)),
            ("model", Type::CUtf8),
            ("unknown_string", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("BaseList"), [
            ("character", Type::I16),
            ("lines", Type::U16),
            ("elements", Type::Repeat(8, Box::new(Type::U16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("OrbLineList"), [
            ("character", Type::I16),
            ("line", Type::U16),
            ("slot_ids", Type::Repeat(8, Box::new(Type::I16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("bgm"), [
            ("id", Type::U16),
            ("track", Type::CUtf8),
            ("unknown_data", Type::X(2)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("CardTableData"), [
            ("id", Type::I16),
            ("unknown_string", Type::CUtf8),
            ("unknown_data", Type::X(26)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("char_revise"), [
            ("char", Type::CUtf8),
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("unknown_short_3", Type::U16),
            ("unknown_short_4", Type::U16),
            ("unknown_short_5", Type::U16),
            ("unknown_short_6", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("dlc"), [
            ("id", Type::I16),
            ("unknown_text_1", Type::CUtf8),
            ("unknown_text_2", Type::CUtf8),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("FaceSupplementTableData"), [
            ("character", Type::I16),
            ("unknown_data_1", Type::X(16)),
            ("uknown_string", Type::CUtf8),
            ("unknown_data_2", Type::X(8)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("FootStepData"), [
            ("unknown_text", Type::CUtf8),
            ("unknown_byte", Type::U8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("growth"), [
            ("character", Type::I16),
            ("unknown_short_0", Type::I16),
            ("hp_base", Type::I16),
            ("hp_growth", Type::F32),
            ("str_base", Type::I16),
            ("str_growth", Type::F32),
            ("def_base", Type::I16),
            ("def_growth", Type::F32),
            ("ats_base", Type::I16),
            ("ats_growth", Type::F32),
            ("adf_base", Type::I16),
            ("adf_growth", Type::F32),
            ("spd_base", Type::I16),
            ("spd_growth", Type::F32),
            ("dex_base", Type::I16),
            ("dex_growth", Type::F32),
            ("agl_base", Type::I16),
            ("agl_growth", Type::F32),
            ("mov_base", Type::I16),
            ("mov_growth", Type::F32),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("hkitugi_lst"), [
            ("unknown_short", Type::U16),
            ("unknown_byte_1", Type::U8),
            ("unknown_byte_2", Type::U8),
            ("cost", Type::U8),
            ("unknown_byte_3", Type::U8),
            ("text", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("item"), [
            ("id", Type::U16),
            ("character_restriction", Type::U16),
            ("flags", Type::CUtf8),
            ("type", Type::U8),
            ("element", Type::U8),
            ("subtype", Type::U8), // target type for weapons, quartz type (normal/debuff/status quartz) for quartzes
            ("range", Type::U8),
            ("area", Type::U8),
            ("effects", Type::Repeat(2, Box::new(Type::Ref(SmolStr::new("effect"))))),
            ("str", Type::I16),
            ("def", Type::I16),
            ("ats", Type::I16),
            ("adf", Type::I16),
            ("acc", Type::I16),
            ("eva", Type::I16),
            ("spd", Type::I16),
            ("mov", Type::I16),
            ("hp", Type::I16),
            ("ep", Type::I16),
            ("price", Type::U32),
            ("carry_limit", Type::U8),
            ("sort_id", Type::U8),
            ("unknown_byte_1", Type::U8),
            ("unknown_byte_2", Type::U8),
            ("unknown_byte_3", Type::U8),
            ("unknown_short", Type::I16),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("LinkAbList"), [
            ("unknown_data", Type::X(5)),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MG02Text"), [
            ("character", Type::I16),
            ("id", Type::U16),
            ("unknown_short", Type::U16),
            ("text", Type::CUtf8),
            ("unknown_data", Type::X(6)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("magic"), [
            ("id", Type::U16),
            ("character_restriction", Type::U16),
            ("flags", Type::CUtf8),
            ("category", Type::U8),
            ("type", Type::U8),
            ("element", Type::U8),
            ("target_type", Type::U8),
            ("target_range", Type::U8),
            ("target_size", Type::U8),
            ("effects", Type::Repeat(2, Box::new(Type::Ref(SmolStr::new("effect"))))),
            ("cast_delay", Type::U8),
            ("recovery_delay", Type::U8),
            ("cost", Type::U16),
            ("unknown_byte_1", Type::U8),
            ("level_learn", Type::I8),
            ("sort_id", Type::U16),
            ("animation", Type::CUtf8),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MapJumpData"), [
            ("unknown_data_1", Type::X(4)),
            ("name", Type::CUtf8),
            ("unknown_data_2", Type::X(4)),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_string_2", Type::CUtf8),
            ("unknown_data_3", Type::X(28)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MasterQuartzBase"), [
            ("id", Type::I16),
            ("unknown_byte_1", Type::U8),
            ("unknown_byte_2", Type::U8),
            ("unknown_byte_3", Type::U8),
            ("unknown_byte_4", Type::U8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MasterQuartzData"), [
            ("id", Type::I16),
            ("level", Type::U16),
            ("hp", Type::I16),
            ("ep", Type::I16),
            ("str", Type::I16),
            ("def", Type::I16),
            ("ats", Type::I16),
            ("adf", Type::I16),
            ("spd", Type::I16),
            ("effect_data", Type::Repeat(6, Box::new(Type::F32))),
            ("arts", Type::Repeat(2, Box::new(Type::I16))),
            ("memos", Type::Repeat(9, Box::new(Type::I16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MasterQuartzMemo"), [
            ("item_id", Type::I16),
            ("memo_id", Type::I16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("NameTableData"), [
            ("character", Type::U16),
            ("name", Type::CUtf8),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_string_2", Type::CUtf8),
            ("unknown_string_3", Type::CUtf8),
            ("unknown_string_4", Type::CUtf8),
            ("unknown_string_5", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("NaviTextData"), [
            ("unknown_short", Type::U16),
            ("text", Type::CUtf8),
            ("unknown_data_2", Type::X(4)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("PlaceTableData"), [
            ("unknown_short_1", Type::I16),
            ("unknown_short_2", Type::I16),
            ("map", Type::CUtf8),
            ("name", Type::CUtf8),
            ("unknown_string", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSBook"), [
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("title", Type::CUtf8),
            ("unknown_text_1", Type::CUtf8),
            ("unknown_text_2", Type::CUtf8),
            ("unknown_data", Type::X(2)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSChapter"), [
            ("id", Type::I16),
            ("chapter", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSBookScp"), [
            ("unknown_text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSChar"), [
            ("character", Type::U16),
            ("unknown_data", Type::X(4)),
            ("name_short", Type::CUtf8),
            ("name_long", Type::CUtf8),
            ("affiliation_short", Type::CUtf8),
            ("note_1", Type::CUtf8),
            ("note_2_title", Type::CUtf8),
            ("note_2", Type::CUtf8),
            ("note_3_title", Type::CUtf8),
            ("note_3", Type::CUtf8),
            ("note_4_title", Type::CUtf8),
            ("note_4", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSCook"), [
            ("regular_name", Type::CUtf8),
            ("unknown_data_1", Type::X(34)),
            ("superb", Type::Ref(SmolStr::new("QSCookVariantDescription"))),
            ("regular", Type::Ref(SmolStr::new("QSCookVariantDescription"))),
            ("peculiar", Type::Ref(SmolStr::new("QSCookVariantDescription"))),
            ("unique", Type::Ref(SmolStr::new("QSCookVariantDescription"))),
            ("unknown_data_2", Type::X(11)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSFish"), [
            ("id", Type::U16),
            ("name", Type::CUtf8),
            ("unknown_data_1", Type::X(96)),
            ("description", Type::CUtf8),
            ("unknown_data_2", Type::X(62)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSHelp"), [
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSMons"), [
            ("enemy", Type::CUtf8),
            ("unknown_short_1", Type::I16),
            ("unknown_short_2", Type::I16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSText"), [
            ("id", Type::U16),
            ("unknown_byte_1", Type::U8),
            ("text", Type::CUtf8),
            ("unknown_byte_2", Type::U8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSTitle"), [
            ("unknown_short", Type::I16),
            ("unknown_byte", Type::U8),
            ("title", Type::CUtf8),
            ("persons", Type::CUtf8),
            ("unknown_data", Type::X(9)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QuartzCost"), [
            ("id", Type::I16),
            ("earth_sepith_cost", Type::U16),
            ("water_sepith_cost", Type::U16),
            ("fire_sepith_cost", Type::U16),
            ("wind_sepith_cost", Type::U16),
            ("time_sepith_cost", Type::U16),
            ("space_sepith_cost", Type::U16),
            ("mirage_sepith_cost", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("ShopTitle"), [
            ("unknown_short", Type::I16),
            ("unknown_byte", Type::U8),
            ("name", Type::CUtf8),
            ("unknown_data", Type::X(8)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("SlotCost"), [
            ("slot_position", Type::I16),
            ("earth_sepith_cost", Type::U16),
            ("water_sepith_cost", Type::U16),
            ("fire_sepith_cost", Type::U16),
            ("wind_sepith_cost", Type::U16),
            ("time_sepith_cost", Type::U16),
            ("space_sepith_cost", Type::U16),
            ("mirage_sepith_cost", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("SlotEp"), [
            ("character", Type::I16),
            ("ep_base", Type::U16),
            ("ep_increase", Type::Repeat(8, Box::new(Type::I16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("status"), [
            ("script", Type::CUtf8),
            ("texture", Type::CUtf8),
            ("model", Type::CUtf8),
            ("model_scale", Type::F32),
            ("camera_height", Type::F32),
            ("hitbox_size", Type::F32),
            ("unknown_data_1", Type::X(4)),
            ("unknown_float_4", Type::F32),
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("is_female", Type::U8),
            ("level", Type::U8),
            ("hp", Type::U32),
            ("hp_start", Type::U32),
            ("ep", Type::U16),
            ("ep_start", Type::U16),
            ("cp", Type::U16),
            ("cp_start", Type::U16),
            ("str", Type::U16),
            ("def", Type::U16),
            ("ats", Type::U16),
            ("adf", Type::U16),
            ("unknown_short_7", Type::U16),
            ("unknown_short_8", Type::U16),
            ("spd", Type::U16),
            ("unknown_short_10", Type::U16),
            ("exp", Type::U16),
            ("earth_efficacy", Type::U8),
            ("water_efficacy", Type::U8),
            ("fire_efficacy", Type::U8),
            ("wind_efficacy", Type::U8),
            ("time_efficacy", Type::U8),
            ("space_efficacy", Type::U8),
            ("mirage_efficacy", Type::U8),
            ("psn_efficacy", Type::U8),
            ("seal_efficacy", Type::U8),
            ("mute_efficacy", Type::U8),
            ("blnd_efficacy", Type::U8),
            ("slp_efficacy", Type::U8),
            ("burn_efficacy", Type::U8),
            ("frz_efficacy", Type::U8),
            ("petr_efficacy", Type::U8),
            ("fnt_efficacy", Type::U8),
            ("conf_efficacy", Type::U8),
            ("dblw_efficacy", Type::U8),
            ("nmr_efficacy", Type::U8),
            ("dlay_efficacy", Type::U8),
            ("vnsh_efficacy", Type::U8),
            ("s_down_efficacy", Type::U8),
            ("slash_efficacy", Type::U16),
            ("thrust_efficacy", Type::U16),
            ("pierce_efficacy", Type::U16),
            ("strike_efficacy", Type::U16),
            ("earth_sepith", Type::U8),
            ("water_sepith", Type::U8),
            ("fire_sepith", Type::U8),
            ("wind_sepith", Type::U8),
            ("time_sepith", Type::U8),
            ("space_sepith", Type::U8),
            ("mirage_sepith", Type::U8),
            ("sepith_mass", Type::U8),
            ("drop_1_id", Type::I16),
            ("drop_1_chance", Type::U8),
            ("drop_2_id", Type::I16),
            ("drop_2_chance", Type::U8),
            ("monster_variation_min", Type::F32),
            ("monster_variation_max", Type::F32),
            ("flags", Type::CUtf8),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("status_revise"), [
            ("difficulty", Type::U16),
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("unknown_short_3", Type::U16),
            ("unknown_short_4", Type::U16),
            ("unknown_short_5", Type::U16),
            ("unknown_short_6", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("TacticalBonus"), [
            ("id", Type::U16),
            ("name", Type::CUtf8),
            ("bonus", Type::F32),
            ("unknown_data", Type::X(2)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("TextTableData"), [
            ("id", Type::U16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("text_lst"), [
            ("id", Type::U16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("voice"), [
            ("id", Type::U16),
            ("unknown_byte", Type::U8),
            ("clip", Type::CUtf8),
            ("unknown_short", Type::U16),
            ("unknown_float", Type::F32),
            ("unknown_data", Type::X(8)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("weapon_attribute"), [
            ("character", Type::I16),
            ("unknown_byte_1", Type::U8),
            ("unknown_byte_2", Type::U8),
            ("unknown_byte_3", Type::U8),
            ("unknown_byte_4", Type::U8),
            ("unknown_float_1", Type::F32),
            ("unknown_float_2", Type::F32),
            ("unknown_float_3", Type::F32),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
    ].into_iter().collect();
    pub static ref SCHEMAS_CS1_COMMON : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = [
        (SmolStr::new("effect"), [
            ("id", Type::U8),
            ("data", Type::Repeat(2, Box::new(Type::I16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSCookVariantDescription"), [
            ("item_id", Type::I16),
            ("description_1", Type::CUtf8),
            ("description_2", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
    ].into_iter().collect();

    pub static ref SCHEMAS_CS2 : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = [
    ].into_iter().collect();
    pub static ref SCHEMAS_CS2_COMMON : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = [
    ].into_iter().collect();

    pub static ref SCHEMAS_CS3 : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = [
        (SmolStr::new("ActiveVoiceTableData"), [
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("unknown_short_3", Type::U16),
            ("portrait", Type::CUtf8),
            ("id", Type::U16),
            ("character", Type::I16),
            ("text", Type::CUtf8),
            ("unknown_float_1", Type::F32),
            ("unknown_float_2", Type::F32),
            ("unknown_data", Type::X(12)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("AssetCache"), [
            ("unknown_data", Type::X(2)),
            ("unknown_string", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("AtBonus"), [
            ("unknown_int", Type::U32),
            ("inverse_chance", Type::U8),
            ("hp_10_perc", Type::U8),
            ("hp_30_perc", Type::U8),
            ("hp_50_perc", Type::U8),
            ("ep_10_perc", Type::U8),
            ("ep_30_perc", Type::U8),
            ("ep_50_perc", Type::U8),
            ("cp_10", Type::U8),
            ("cp_30", Type::U8),
            ("cp_50", Type::U8),
            ("cp_100", Type::U8),
            ("bp_1", Type::U8),
            ("bp_3", Type::U8),
            ("bp_5", Type::U8),
            ("crit", Type::U8),
            ("sepith_up", Type::U8),
            ("zero_art", Type::U8),
            ("zero_order", Type::U8),
            ("guard_break", Type::U8),
            ("status_attack", Type::U8),
            ("deathblow", Type::U8),
            ("vanish", Type::U8),
            ("cp_200", Type::U8),
            ("cp_0", Type::U8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("AttachTableData"), [
            ("character", Type::U16),
            ("unknown_data", Type::X(20)),
            ("model", Type::CUtf8),
            ("unknown_string", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("BaseList"), [
            ("character", Type::I16),
            ("lines", Type::U16),
            ("elements", Type::Repeat(7, Box::new(Type::U16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("OrbLineList"), [
            ("character", Type::I16),
            ("line", Type::U16),
            ("slot_ids", Type::Repeat(7, Box::new(Type::I16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("bgm"), [
            ("id", Type::U16),
            ("track", Type::CUtf8),
            ("unknown_data", Type::X(2)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("char_revise"), [
            ("char", Type::CUtf8),
            ("hp", Type::U16),
            ("str", Type::U16),
            ("def", Type::U16),
            ("ats", Type::U16),
            ("adf", Type::U16),
            ("spd", Type::U16),
            ("brk", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("CharData"), [
            ("unknown_shorts", Type::Repeat(3, Box::new(Type::U16))),
            ("unknown_floats", Type::Repeat(5, Box::new(Type::F32))),
            ("unknown_data", Type::X(12)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("CompHelpData"), [
            ("type", Type::U16),
            ("text", Type::CUtf8),
            ("value", Type::I32),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("EventTableData"), [
            ("id", Type::U16),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_string_2", Type::CUtf8),
            ("unknown_data_1", Type::X(6)),
            ("unknown_string_3", Type::CUtf8),
            ("unknown_data_2", Type::X(18)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("FootStepData"), [
            ("unknown_string", Type::CUtf8),
            ("unknown_byte", Type::U8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("game_difficulty"), [
            ("difficulty", Type::U16),
            ("modifier", Type::I16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("hkitugi_lst"), [
            ("unknown_short", Type::U16),
            ("unknown_data", Type::X(2)),
            ("text", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("item"), [
            ("id", Type::U16),
            ("character_restriction", Type::U16),
            ("flags", Type::CUtf8),
            ("type", Type::U16),
            ("subtype", Type::U16),
            ("element", Type::U16),
            ("slash", Type::U8),
            ("thrust", Type::U8),
            ("pierce", Type::U8),
            ("strike", Type::U8),
            ("target_type", Type::U8),
            ("range", Type::F32),
            ("area", Type::U8),
            ("unknown_float_1", Type::F32),
            ("unknown_float_2", Type::F32),
            ("unknown_float_3", Type::F32),
            ("effects", Type::Repeat(5, Box::new(Type::Ref(SmolStr::new("effect"))))),
            ("str", Type::I16),
            ("def", Type::I16),
            ("ats", Type::I16),
            ("adf", Type::I16),
            ("acc", Type::I16),
            ("eva", Type::I16),
            ("spd", Type::I16),
            ("mov", Type::I16),
            ("hp", Type::I16),
            ("ep", Type::I16),
            ("price", Type::U32),
            ("carry_limit", Type::U8),
            ("sort_id", Type::U16),
            ("unknown_short", Type::U16),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
            ("unknown_data", Type::X(8)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("item_q"), [
            ("id", Type::U16),
            ("unknown_short_1", Type::U16),
            ("flags", Type::CUtf8),
            ("type", Type::U16),
            ("subtype", Type::U16),
            ("element", Type::U16),
            ("unknown_data_1", Type::X(4)),
            ("quartz_type", Type::U8),
            ("rating", Type::F32),
            ("unknown_byte_1", Type::U8),
            ("unknown_data_2", Type::X(12)),
            ("effects", Type::Repeat(5, Box::new(Type::Ref(SmolStr::new("effect"))))),
            ("str", Type::I16),
            ("def", Type::I16),
            ("ats", Type::I16),
            ("adf", Type::I16),
            ("acc", Type::I16),
            ("eva", Type::I16),
            ("spd", Type::I16),
            ("mov", Type::I16),
            ("hp", Type::I16),
            ("ep", Type::I16),
            ("price", Type::U32),
            ("carry_limit", Type::U8),
            ("sort_id", Type::U16),
            ("unknown_short_2", Type::U16),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
            ("prio_balanced", Type::U16),
            ("prio_physical", Type::U16),
            ("prio_magical", Type::U16),
            ("prio_speed", Type::U16),
            ("arts", Type::Repeat(6, Box::new(Type::I16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("ItemHelpData"), [
            ("effect", Type::U16),
            ("text", Type::CUtf8),
            ("unknown_data", Type::X(9)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("KisinPilotData"), [
            ("unknown_data", Type::X(8)),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_string_2", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("LinkAbText"), [
            ("id", Type::U8),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("magic"), [
            ("id", Type::U16),
            ("character_restriction", Type::U16),
            ("flags", Type::CUtf8),
            ("category", Type::U8),
            ("type", Type::U8),
            ("element", Type::U8),
            ("juna_specific", Type::U8),
            ("target_type", Type::U8),
            ("target_range", Type::F32),
            ("target_size", Type::U8),
            ("unknown_float_1", Type::F32),
            ("unknown_float_2", Type::F32),
            ("unknown_float_3", Type::F32),
            ("effects", Type::Repeat(5, Box::new(Type::Ref(SmolStr::new("effect"))))),
            ("cast_delay", Type::U8),
            ("recovery_delay", Type::U8),
            ("cost", Type::U16),
            ("unbalance_bonus", Type::U8),
            ("break_bonus", Type::U16),
            ("level_learn", Type::U8),
            ("sort_id", Type::U16),
            ("animation", Type::CUtf8),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("magicbo"), [
            ("id", Type::U16),
            ("effects", Type::Repeat(5, Box::new(Type::Ref(SmolStr::new("effect"))))),
            ("turns", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MapJumpData"), [
            ("unknown_data_1", Type::X(5)),
            ("name", Type::CUtf8),
            ("unknown_data_2", Type::X(4)),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_string_2", Type::CUtf8),
            ("unknown_data_3", Type::X(24)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MarkerTableData"), [
            ("unknown_data_1", Type::X(6)),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_data_2", Type::X(9)),
            ("unknown_string_2", Type::CUtf8),
            ("unknown_data_3", Type::X(12)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MasterQuartzBase"), [
            ("item_id", Type::U16),
            ("mq_id", Type::U16),
            ("unknown_short", Type::U16),
            ("hp_pattern", Type::U16),
            ("ep_pattern", Type::U16),
            ("str_pattern", Type::U16),
            ("def_pattern", Type::U16),
            ("ats_pattern", Type::U16),
            ("adf_pattern", Type::U16),
            ("spd_pattern", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MasterQuartzData"), [
            ("item_id", Type::U16),
            ("level", Type::U16),
            ("effect_data", Type::Repeat(9, Box::new(Type::F32))),
            ("arts", Type::Repeat(2, Box::new(Type::I16))),
            ("memos", Type::Repeat(9, Box::new(Type::I16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MasterQuartzDummy"), [
            ("dummy_id", Type::I16),
            ("original_id", Type::I16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MasterQuartzMemo"), [
            ("item_id", Type::I16),
            ("memo_id", Type::I16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MasterQuartzStatus"), [
            ("pattern_id", Type::U8),
            ("level", Type::U16),
            ("main_hp", Type::I16),
            ("main_ep", Type::I16),
            ("main_str", Type::I16),
            ("main_def", Type::I16),
            ("main_ats", Type::I16),
            ("main_adf", Type::I16),
            ("main_spd", Type::I16),
            ("sub_hp", Type::I16),
            ("sub_ep", Type::I16),
            ("sub_str", Type::I16),
            ("sub_def", Type::I16),
            ("sub_ats", Type::I16),
            ("sub_adf", Type::I16),
            ("sub_spd", Type::I16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MG08Card"), [
            ("id", Type::U16),
            ("type", Type::U16),
            ("element", Type::U16),
            ("name", Type::CUtf8),
            ("item", Type::U16),
            ("mana", Type::U16),
            ("cost", Type::U16),
            ("atk", Type::U16),
            ("hp", Type::U16),
            ("unknown_int", Type::U16),
            ("ranged_flag", Type::U16),
            ("ranged_string", Type::CUtf8),
            ("flight_flag", Type::U16),
            ("flight_string", Type::CUtf8),
            ("skill_id", Type::U16),
            ("skill_string", Type::CUtf8),
            ("skill_cost", Type::U16),
            ("skill_arg", Type::U16),
            ("skill_description", Type::CUtf8),
            ("ability_id", Type::U16),
            ("ability_arg", Type::U16),
            ("ability_description", Type::CUtf8),
            ("effect_id", Type::U16),
            ("unknown_int_2", Type::U16),
            ("effect_arg_1", Type::I16),
            ("effect_arg_2", Type::I16),
            ("effect_description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MG08CharMsg"), [
            ("character", Type::U16),
            ("id", Type::U16),
            ("unknown_int", Type::U16),
            ("text", Type::CUtf8),
            ("voice", Type::U16),
            ("unknown_data", Type::X(4)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MG08EnemyAi"), [
            ("character", Type::U16),
            ("id", Type::U16),
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("unknown_short_3", Type::U16),
            ("unknown_short_4", Type::U16),
            ("master", Type::U16),
            ("cards", Type::Repeat(20, Box::new(Type::U16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MG08PlyMsg"), [
            ("character", Type::U16),
            ("id", Type::U16),
            ("unknown_int", Type::U16),
            ("text", Type::CUtf8),
            ("voice", Type::U16),
            ("unknown_data", Type::X(4)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MG08PresetDeck"), [
            ("id", Type::U8),
            ("master", Type::U8),
            ("cards", Type::Repeat(20, Box::new(Type::U8))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("MG08Text"), [
            ("id", Type::U16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("NameTableData"), [
            ("character", Type::U16),
            ("name", Type::CUtf8),
            ("model", Type::CUtf8),
            ("animation", Type::CUtf8),
            ("default_face", Type::CUtf8),
            ("face", Type::CUtf8),
            ("unknown_string", Type::CUtf8),
            ("unknown_data_1", Type::X(4)),
            ("unknown_floats", Type::Repeat(2, Box::new(Type::F32))),
            ("unknown_shorts", Type::Repeat(2, Box::new(Type::I16))),
            ("unknown_data_2", Type::X(3)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("NameVis"), [
            ("unknown_string", Type::CUtf8),
            ("unknown_data", Type::X(8)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("NaviTextData"), [
            ("unknown_data_1", Type::X(4)),
            ("text", Type::CUtf8),
            ("unknown_data_2", Type::X(7)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("PlaceTableData"), [
            ("unknown_data_1", Type::X(4)),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_data_2", Type::X(4)),
            ("unknown_string_2", Type::CUtf8),
            ("unknown_string_3", Type::CUtf8),
            ("unknown_string_4", Type::CUtf8),
            ("unknown_string_5", Type::CUtf8),
            ("unknown_string_6", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("PopupMsgFaceCharData"), [
            ("unknown_short", Type::U16),
            ("unknown_string", Type::CUtf8),
            ("unknown_data", Type::X(44)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSBook"), [
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("title", Type::CUtf8),
            ("unknown_text_1", Type::CUtf8),
            ("unknown_text_2", Type::CUtf8),
            ("unknown_data", Type::X(5)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSChapter"), [
            ("section_id", Type::U16),
            ("section_name", Type::CUtf8),
            ("unknown_short", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSChar"), [
            ("character", Type::U16),
            ("unknown_data", Type::X(6)),
            ("unknown_text_1", Type::CUtf8),
            ("name_short", Type::CUtf8),
            ("name_long", Type::CUtf8),
            ("affiliation_short", Type::CUtf8),
            ("affiliation_long", Type::CUtf8),
            ("note_1", Type::CUtf8),
            ("note_2_title", Type::CUtf8),
            ("note_2", Type::CUtf8),
            ("note_3_title", Type::CUtf8),
            ("note_3", Type::CUtf8),
            ("note_4_title", Type::CUtf8),
            ("note_4", Type::CUtf8),
            ("note_5_title", Type::CUtf8),
            ("note_5", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSCook"), [
            ("regular_name", Type::CUtf8),
            ("unknown_data_1", Type::X(34)),
            ("superb", Type::Ref(SmolStr::new("QSCookVariantDescription"))),
            ("regular", Type::Ref(SmolStr::new("QSCookVariantDescription"))),
            ("peculiar", Type::Ref(SmolStr::new("QSCookVariantDescription"))),
            ("unique", Type::Ref(SmolStr::new("QSCookVariantDescription"))),
            ("unknown_data_2", Type::X(48)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSFish"), [
            ("id", Type::U16),
            ("unknown_string", Type::CUtf8),
            ("name", Type::CUtf8),
            ("unknown_data", Type::X(167)),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSHelp"), [
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSTitle"), [
            ("unknown_short", Type::U16),
            ("unknown_byte", Type::U8),
            ("title", Type::CUtf8),
            ("persons", Type::CUtf8),
            ("unknown_data", Type::X(13)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QuartzCost"), [
            ("id", Type::I16),
            ("earth_sepith_cost", Type::U16),
            ("water_sepith_cost", Type::U16),
            ("fire_sepith_cost", Type::U16),
            ("wind_sepith_cost", Type::U16),
            ("time_sepith_cost", Type::U16),
            ("space_sepith_cost", Type::U16),
            ("mirage_sepith_cost", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSMons"), [
            ("enemy", Type::CUtf8),
            ("unknown_string", Type::CUtf8),
            ("section_id", Type::U16),
            ("unknown_short", Type::U16),
            ("is_extra", Type::U8),
            ("unknown_byte", Type::U8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSRank"), [
            ("id", Type::U16),
            ("text", Type::CUtf8),
            ("unknown_data", Type::X(6)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSText"), [
            ("id", Type::U16),
            ("unknown_byte_1", Type::U8),
            ("text", Type::CUtf8),
            ("unknown_byte_2", Type::U8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSTitle"), [
            ("unknown_short", Type::U16),
            ("unknown_byte", Type::U8),
            ("title", Type::CUtf8),
            ("persons", Type::CUtf8),
            ("unknown_data", Type::X(13)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QuartzCost"), [
            ("id", Type::I16),
            ("earth_sepith_cost", Type::U16),
            ("water_sepith_cost", Type::U16),
            ("fire_sepith_cost", Type::U16),
            ("wind_sepith_cost", Type::U16),
            ("time_sepith_cost", Type::U16),
            ("space_sepith_cost", Type::U16),
            ("mirage_sepith_cost", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("SchoolGrade"), [
            ("id", Type::U16),
            ("text", Type::CUtf8),
            ("unknown_data", Type::X(6)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("se"), [
            ("id", Type::U16),
            ("unknown_string_1", Type::CUtf8),
            ("unknown_string_2", Type::CUtf8),
            ("unknown_data_1", Type::X(2)),
            ("unknown_float_1", Type::F32),
            ("unknown_float_2", Type::F32),
            ("unknown_data_2", Type::X(4)),
            ("unknown_float_3", Type::F32),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("SlotCost"), [
            ("slot_position", Type::I16),
            ("level", Type::I16),
            ("earth_sepith_cost", Type::U16),
            ("water_sepith_cost", Type::U16),
            ("fire_sepith_cost", Type::U16),
            ("wind_sepith_cost", Type::U16),
            ("time_sepith_cost", Type::U16),
            ("space_sepith_cost", Type::U16),
            ("mirage_sepith_cost", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("SlotEp"), [
            ("character", Type::I16),
            ("slot_level", Type::I16),
            ("ep_base", Type::U16),
            ("ep_increase", Type::Repeat(7, Box::new(Type::I16))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("status"), [
            ("script", Type::CUtf8),
            ("texture", Type::CUtf8),
            ("model", Type::CUtf8),
            ("model_scale", Type::F32),
            ("camera_pivot_height", Type::F32),
            ("hitbox_size", Type::F32),
            ("unknown_float_1", Type::F32),
            ("unknown_float_2", Type::F32),
            ("unknown_float_3", Type::F32),
            ("unknown_float_4", Type::F32),
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("is_female", Type::U8),
            ("level", Type::U8),
            ("hp_base", Type::U32),
            ("hp_growth", Type::F32),
            ("max_ep", Type::U16),
            ("start_ep", Type::U16),
            ("max_cp", Type::U16),
            ("start_cp", Type::U16),
            ("str_base", Type::U16),
            ("str_growth", Type::F32),
            ("def_base", Type::U16),
            ("def_growth", Type::F32),
            ("ats_base", Type::U16),
            ("ats_growth", Type::F32),
            ("adf_base", Type::U16),
            ("adf_growth", Type::F32),
            ("dex_base", Type::U16),
            ("dex_growth", Type::F32),
            ("agl_base", Type::U16),
            ("agl_growth", Type::F32),
            ("eva", Type::U16),
            ("spd_base", Type::U16),
            ("spd_growth", Type::F32),
            ("mov_base", Type::U16),
            ("mov_growth", Type::F32),
            ("exp_base", Type::U16),
            ("exp_growth", Type::F32),
            ("brk_mult_base", Type::U16),
            ("brk_mult_growth", Type::F32),
            ("earth_efficacy", Type::U8),
            ("water_efficacy", Type::U8),
            ("fire_efficacy", Type::U8),
            ("wind_efficacy", Type::U8),
            ("time_efficacy", Type::U8),
            ("space_efficacy", Type::U8),
            ("mirage_efficacy", Type::U8),
            ("psn_efficacy", Type::U8),
            ("seal_efficacy", Type::U8),
            ("mute_efficacy", Type::U8),
            ("blnd_efficacy", Type::U8),
            ("slp_efficacy", Type::U8),
            ("burn_efficacy", Type::U8),
            ("frz_efficacy", Type::U8),
            ("petr_efficacy", Type::U8),
            ("fnt_efficacy", Type::U8),
            ("conf_efficacy", Type::U8),
            ("chrm_efficacy", Type::U8),
            ("dblw_efficacy", Type::U8),
            ("nmr_efficacy", Type::U8),
            ("dlay_efficacy", Type::U8),
            ("vnsh_efficacy", Type::U8),
            ("s_dwn_efficacy", Type::U8),
            ("slash_efficacy", Type::U16),
            ("thrust_efficacy", Type::U16),
            ("pierce_efficacy", Type::U16),
            ("strike_efficacy", Type::U16),
            ("earth_sepith_base", Type::U8),
            ("water_sepith_base", Type::U8),
            ("fire_sepith_base", Type::U8),
            ("wind_sepith_base", Type::U8),
            ("time_sepith_base", Type::U8),
            ("space_sepith_base", Type::U8),
            ("mirage_sepith_base", Type::U8),
            ("sepith_mass_base", Type::U8),
            ("earth_sepith_growth", Type::F32),
            ("water_sepith_growth", Type::F32),
            ("fire_sepith_growth", Type::F32),
            ("wind_sepith_growth", Type::F32),
            ("time_sepith_growth", Type::F32),
            ("space_sepith_growth", Type::F32),
            ("mirage_sepith_growth", Type::F32),
            ("sepith_mass_growth", Type::F32),
            ("drop_id_1", Type::U16),
            ("drop_chance_1", Type::U8),
            ("drop_id_2", Type::U16),
            ("drop_chance_2", Type::U8),
            ("stat_variation_min", Type::F32),
            ("stat_variation_max", Type::F32),
            ("flags", Type::CUtf8),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("status_p"), [
            ("id", Type::U16),
            ("script", Type::CUtf8),
            ("texture", Type::CUtf8),
            ("model", Type::CUtf8),
            ("model_scale", Type::F32),
            ("camera_pivot_height", Type::F32),
            ("hitbox_size", Type::F32),
            ("unknown_float_1", Type::F32),
            ("unknown_float_2", Type::F32),
            ("unknown_float_3", Type::F32),
            ("unknown_float_4", Type::F32),
            ("unknown_short_1", Type::U16),
            ("unknown_short_2", Type::U16),
            ("is_female", Type::U8),
            ("level", Type::U8),
            ("hp_base", Type::U32),
            ("hp_growth", Type::F32),
            ("max_ep", Type::U16),
            ("start_ep", Type::U16),
            ("max_cp", Type::U16),
            ("start_cp", Type::U16),
            ("str_base", Type::U16),
            ("str_growth", Type::F32),
            ("def_base", Type::U16),
            ("def_growth", Type::F32),
            ("ats_base", Type::U16),
            ("ats_growth", Type::F32),
            ("adf_base", Type::U16),
            ("adf_growth", Type::F32),
            ("dex_base", Type::U16),
            ("dex_growth", Type::F32),
            ("agl_base", Type::U16),
            ("agl_growth", Type::F32),
            ("eva", Type::U16),
            ("spd_base", Type::U16),
            ("spd_growth", Type::F32),
            ("mov_base", Type::U16),
            ("mov_growth", Type::F32),
            ("exp_base", Type::U16),
            ("exp_growth", Type::F32),
            ("brk_mult_base", Type::U16),
            ("brk_mult_growth", Type::F32),
            ("earth_efficacy", Type::U8),
            ("water_efficacy", Type::U8),
            ("fire_efficacy", Type::U8),
            ("wind_efficacy", Type::U8),
            ("time_efficacy", Type::U8),
            ("space_efficacy", Type::U8),
            ("mirage_efficacy", Type::U8),
            ("psn_efficacy", Type::U8),
            ("seal_efficacy", Type::U8),
            ("mute_efficacy", Type::U8),
            ("blnd_efficacy", Type::U8),
            ("slp_efficacy", Type::U8),
            ("burn_efficacy", Type::U8),
            ("frz_efficacy", Type::U8),
            ("petr_efficacy", Type::U8),
            ("fnt_efficacy", Type::U8),
            ("conf_efficacy", Type::U8),
            ("chrm_efficacy", Type::U8),
            ("dblw_efficacy", Type::U8),
            ("nmr_efficacy", Type::U8),
            ("dlay_efficacy", Type::U8),
            ("vnsh_efficacy", Type::U8),
            ("s_dwn_efficacy", Type::U8),
            ("slash_efficacy", Type::U16),
            ("thrust_efficacy", Type::U16),
            ("pierce_efficacy", Type::U16),
            ("strike_efficacy", Type::U16),
            ("earth_sepith_base", Type::U8),
            ("water_sepith_base", Type::U8),
            ("fire_sepith_base", Type::U8),
            ("wind_sepith_base", Type::U8),
            ("time_sepith_base", Type::U8),
            ("space_sepith_base", Type::U8),
            ("mirage_sepith_base", Type::U8),
            ("sepith_mass_base", Type::U8),
            ("earth_sepith_growth", Type::F32),
            ("water_sepith_growth", Type::F32),
            ("fire_sepith_growth", Type::F32),
            ("wind_sepith_growth", Type::F32),
            ("time_sepith_growth", Type::F32),
            ("space_sepith_growth", Type::F32),
            ("mirage_sepith_growth", Type::F32),
            ("sepith_mass_growth", Type::F32),
            ("drop_id_1", Type::U16),
            ("drop_chance_1", Type::U8),
            ("drop_id_2", Type::U16),
            ("drop_chance_2", Type::U8),
            ("stat_variation_min", Type::F32),
            ("stat_variation_max", Type::F32),
            ("flags", Type::CUtf8),
            ("name", Type::CUtf8),
            ("description", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("status_revise"), [
            ("difficulty", Type::U16),
            ("hp", Type::U16),
            ("str", Type::U16),
            ("def", Type::U16),
            ("ats", Type::U16),
            ("adf", Type::U16),
            ("spd", Type::U16),
            ("brk", Type::U16),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("TacticalBonus"), [
            ("id", Type::U16),
            ("name", Type::CUtf8),
            ("bonus", Type::F32),
            ("unknown_data", Type::X(2)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("text_lst"), [
            ("id", Type::U16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("TextTableData"), [
            ("id", Type::U16),
            ("text", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("voice"), [
            ("id", Type::U16),
            ("unknown_byte", Type::U8),
            ("clip", Type::CUtf8),
            ("unknown_short", Type::U16),
            ("unknown_float", Type::F32),
            ("unknown_data", Type::X(12)),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
    ].into_iter().collect();
    pub static ref SCHEMAS_CS3_COMMON : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = [
        (SmolStr::new("effect"), [
            ("id", Type::U16),
            ("data", Type::Repeat(3, Box::new(Type::I32))),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
        (SmolStr::new("QSCookVariantDescription"), [
            ("item_id", Type::I16),
            ("description_1", Type::CUtf8),
            ("description_2", Type::CUtf8),
        ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
    ].into_iter().collect();


    pub static ref SCHEMAS_CS1_ATOMIC : IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>> = get_nf1_for_all(&SCHEMAS_CS1, &SCHEMAS_CS1_COMMON).expect("");
    pub static ref SCHEMAS_CS2_ATOMIC : IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>> = get_nf1_for_all(&SCHEMAS_CS2, &SCHEMAS_CS2_COMMON).expect("");
    pub static ref SCHEMAS_CS3_ATOMIC : IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>> = get_nf1_for_all(&SCHEMAS_CS3, &SCHEMAS_CS3_COMMON).expect("");
}

#[cfg(test)]
mod test {
    use super::{deserialize_tbl_v1, deserialize_tbl_v2, serialize_tbl_v1, serialize_tbl_v2, DeserializeError, WarningLogger, SCHEMAS_CS1_ATOMIC, SCHEMAS_CS2_ATOMIC, SCHEMAS_CS3_ATOMIC};

    use indexmap::IndexMap;
    use smol_str::SmolStr;

    static INIT_LOG: std::sync::Once = std::sync::Once::new();

    #[cfg(feature = "roundtrip-cs1")]
    #[test]
    fn test_roundtrip_cs1_strict() {
        INIT_LOG.call_once(|| {
            simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
        });
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs1/data/text/"));
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(parent) = path.parent() {
                if parent.file_name() == Some(&std::ffi::OsStr::new("dat")) {
                    continue; // TODO: SJIS support for jp files would be nice
                }
            }
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                log::info!("{}", path.display());
                let fixed = crate::cs1::tbl::fix(&std::fs::read(path).unwrap());
                let mut logger = WarningLogger::<DeserializeError>::new();
                let entries = deserialize_tbl_v1(&mut std::io::Cursor::new(&fixed), &mut logger, true, &SCHEMAS_CS1_ATOMIC).unwrap();
                assert!(!logger.has_warnings());
                let serialized = {
                    let mut serialized = std::io::Cursor::new(vec![]);
                    let e: &[(SmolStr, IndexMap<SmolStr, super::AtomicValue>)] = &entries;
                    let _ = serialize_tbl_v1(&mut serialized, &e).unwrap();
                    serialized.into_inner()
                };
                assert_eq!(serialized, fixed);
            }
        }
    }

    #[cfg(feature = "roundtrip-cs1")]
    #[test]
    fn test_parse_cs1_nonstrict() {
        INIT_LOG.call_once(|| {
            simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
        });
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs1/data/text/"));
        // TODO: test that each schema has at least one entry in any file
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(parent) = path.parent() {
                if parent.file_name() == Some(&std::ffi::OsStr::new("dat")) || path.file_name() == Some(&std::ffi::OsStr::new("t_main.tbl")) {
                    continue; // TODO: schemas for the headers with broken lengths
                }
            }
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                log::info!("{}", path.display());
                let mut f = std::io::BufReader::new(std::fs::File::open(path).unwrap());
                let mut logger = WarningLogger::<DeserializeError>::new();
                let entries = deserialize_tbl_v1(&mut f, &mut logger, false, &SCHEMAS_CS1_ATOMIC).unwrap();
                let mut serialized = std::io::Cursor::new(vec![]);
                let e: &[(SmolStr, IndexMap<SmolStr, super::AtomicValue>)] = &entries;
                let _ = serialize_tbl_v1(&mut serialized, &e).unwrap();
                // no comparison here
            }
        }
    }

    #[cfg(feature = "roundtrip-cs2")]
    #[test]
    fn test_roundtrip_cs2_strict() {
        INIT_LOG.call_once(|| {
            simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
        });
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs2/data/text/"));
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(parent) = path.parent() {
                if parent.file_name() == Some(&std::ffi::OsStr::new("dat")) {
                    continue; // TODO: SJIS support for jp files would be nice
                }
            }
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                log::info!("{}", path.display());
                let fixed = std::fs::read(path).unwrap();
                let mut logger = WarningLogger::<DeserializeError>::new();
                let (headers_with_counts, entries) = deserialize_tbl_v2(&mut std::io::Cursor::new(&fixed), &mut logger, true, &SCHEMAS_CS2_ATOMIC).unwrap();
                assert!(!logger.has_warnings());
                let serialized = {
                    let mut serialized = std::io::Cursor::new(vec![]);
                    let e: &[(SmolStr, IndexMap<SmolStr, super::AtomicValue>)] = &entries;
                    let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                    let _ = serialize_tbl_v2(&mut serialized, &headers, &e).unwrap();
                    serialized.into_inner()
                };
                assert_eq!(serialized, fixed);
            }
        }
    }

    #[cfg(feature = "roundtrip-cs2")]
    #[test]
    fn test_parse_cs2_nonstrict() {
        INIT_LOG.call_once(|| {
            simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
        });
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs2/data/text/"));
        // TODO: test that each schema has at least one entry in any file
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(parent) = path.parent() {
                if parent.file_name() == Some(&std::ffi::OsStr::new("dat")) {
                    continue; // TODO: schemas for the headers with broken lengths
                }
            }
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                log::info!("{}", path.display());
                let mut f = std::io::BufReader::new(std::fs::File::open(path).unwrap());
                let mut logger = WarningLogger::<DeserializeError>::new();
                let (headers_with_counts, entries) = deserialize_tbl_v2(&mut f, &mut logger, false, &SCHEMAS_CS2_ATOMIC).unwrap();
                let mut serialized = std::io::Cursor::new(vec![]);
                let e: &[(SmolStr, IndexMap<SmolStr, super::AtomicValue>)] = &entries;
                let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                let _ = serialize_tbl_v2(&mut serialized, &headers, &e).unwrap();
                // no comparison here
            }
        }
    }

    #[cfg(feature = "roundtrip-cs3")]
    #[test]
    fn test_roundtrip_cs3_strict() {
        INIT_LOG.call_once(|| {
            simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
        });
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs3/data/text/"));
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                log::info!("{}", path.display());
                let fixed = crate::cs3::tbl::fix(&std::fs::read(path).unwrap());
                let mut logger = WarningLogger::<DeserializeError>::new();
                let (headers_with_counts, entries) = deserialize_tbl_v2(&mut std::io::Cursor::new(&fixed), &mut logger, true, &SCHEMAS_CS3_ATOMIC).unwrap();
                assert!(!logger.has_warnings());
                let serialized = {
                    let mut serialized = std::io::Cursor::new(vec![]);
                    let e: &[(SmolStr, IndexMap<SmolStr, super::AtomicValue>)] = &entries;
                    let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                    let _ = serialize_tbl_v2(&mut serialized, &headers, &e).unwrap();
                    serialized.into_inner()
                };
                assert_eq!(serialized, fixed);
            }
        }
    }

    #[cfg(feature = "roundtrip-cs3")]
    #[test]
    fn test_parse_cs3_nonstrict() {
        INIT_LOG.call_once(|| {
            simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
        });
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs3/data/text/"));
        // TODO: test that each schema has at least one entry in any file
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                log::info!("{}", path.display());
                let mut f = std::io::BufReader::new(std::fs::File::open(path).unwrap());
                let mut logger = WarningLogger::<DeserializeError>::new();
                let (headers_with_counts, entries) = deserialize_tbl_v2(&mut f, &mut logger, false, &SCHEMAS_CS3_ATOMIC).unwrap();
                let mut serialized = std::io::Cursor::new(vec![]);
                let e: &[(SmolStr, IndexMap<SmolStr, super::AtomicValue>)] = &entries;
                let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                let _ = serialize_tbl_v2(&mut serialized, &headers, &e).unwrap();
                // no comparison here
            }
        }
    }
}
