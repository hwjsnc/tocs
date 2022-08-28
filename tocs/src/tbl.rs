use hex::FromHex as _;
use std::str::FromStr as _;

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
    pub fn new(ignore: bool) -> Self {
        Self {
            ignore,
            warnings: std::vec::Vec::new(),
        }
    }

    pub fn log(&mut self, warning: W) {
        let Self { ignore, warnings } = self;
        if !*ignore {
            warnings.push(warning);
        }
    }

    pub fn ignore_warnings(&self) -> bool {
        self.ignore
    }

    pub fn clear_warnings(&mut self) {
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
    Ref(String),
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

// Atomic values - values corresponding to the atomic types
#[derive(Clone, Debug)]
pub enum AtomicValue<S> {
    I8(i8),
    I16(i16),
    I32(i32),
    U8(u8),
    U16(u16),
    U32(u32),
    F32(f32),
    CUtf8(S),
    X(std::vec::Vec<u8>),
}

// Type -> Atomic Types (unrolling sequences, dereferencing common structures such as effects)

#[derive(Clone, Debug)]
pub enum NF1ErrorDetail {
    NoSuchSchema(String),
    Duplicate(String),
}

#[derive(Clone, Debug)]
pub enum NF1ErrorPosition {
    SchemaLookup,
    Deref(String),
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
pub fn get_nf1(schema_name: &str, schemas: &indexmap::IndexMap<String, indexmap::IndexMap<String, Type>>) -> Result<indexmap::IndexMap<String, AtomicType>, NF1Error> {
    // TODO: check duplicates
    let schema = schemas.get(schema_name).ok_or_else(|| NF1Error {
        detail: NF1ErrorDetail::NoSuchSchema(schema_name.to_owned()),
        context: vec![NF1ErrorPosition::SchemaLookup],
    })?;
    let mut nf1 = indexmap::IndexMap::new();
    for (name, ty) in schema.iter() {
        if let Some(t) = ty.try_to_atomic_type() {
            nf1.insert(name.clone(), t);
            continue;
        }
        match ty {
            Type::Repeat(n, ty) => {
                if let Some(t) = ty.try_to_atomic_type() {
                    for i in 1..=*n {
                        nf1.insert(format!("{} {}", name, i), t.clone());
                    }
                } else if let Type::Ref(s) = ty.as_ref() {
                    todo!()
                } else {
                    panic!("unsupported type for repeat in normal form: {:?}", ty);
                }
            }
            Type::Ref(s) => {
                let sub_nf1 = get_nf1(&s, schemas).map_err(|e| e.add_context(NF1ErrorPosition::Deref(schema_name.to_string())))?;
                for (n, t) in sub_nf1.into_iter() {
                    nf1.insert(format!("{} {}", name, n), t);
                }
            }
            _ => unreachable!("all other types should be primitive and caught by the 'if let' above"),
        }
    }
    Ok(nf1)
}

pub fn get_nf1_for_all(
    schemas: &indexmap::IndexMap<String, indexmap::IndexMap<String, Type>>,
    common: &indexmap::IndexMap<String, indexmap::IndexMap<String, Type>>,
) -> Result<indexmap::IndexMap<String, indexmap::IndexMap<String, AtomicType>>, NF1Error> {
    let mut joint = schemas.clone();
    joint.extend(common.clone().into_iter());
    let joint = joint;
    let mut res = indexmap::IndexMap::new();
    for (name, schema) in schemas {
        res.insert(name.clone(), get_nf1(name, &joint)?);
    }
    Ok(res)
}

// Conversion to and from strings

pub enum AtomicValueFromStringError {
    Generic,
    LengthMismatch { expected: usize, actual: usize },
}

pub fn try_atomic_value_from_str<'a>(t: &AtomicType, s: &'a str) -> Result<AtomicValue<String>, AtomicValueFromStringError> {
    match t {
        AtomicType::I8 => Ok(AtomicValue::I8(i8::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::I16 => Ok(AtomicValue::I16(i16::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::I32 => Ok(AtomicValue::I32(i32::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::U8 => Ok(AtomicValue::U8(u8::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::U16 => Ok(AtomicValue::U16(u16::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::U32 => Ok(AtomicValue::U32(u32::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::F32 => Ok(AtomicValue::F32(f32::from_str(s).map_err(|_| AtomicValueFromStringError::Generic)?)),
        AtomicType::CUtf8 => Ok(AtomicValue::CUtf8(s.to_string())),
        AtomicType::X(n) => {
            let data = std::vec::Vec::<u8>::from_hex(s).map_err(|_| AtomicValueFromStringError::Generic)?;
            if data.len() != *n {
                Err(AtomicValueFromStringError::LengthMismatch { expected: *n, actual: data.len() })
            } else {
                Ok(AtomicValue::X(data))
            }
        }
    }
}

pub fn atomic_value_to_string<S: AsRef<str>>(v: &AtomicValue<S>) -> String {
    match v {
        AtomicValue::I8(x) => x.to_string(),
        AtomicValue::I16(x) => x.to_string(),
        AtomicValue::I32(x) => x.to_string(),
        AtomicValue::U8(x) => x.to_string(),
        AtomicValue::U16(x) => x.to_string(),
        AtomicValue::U32(x) => x.to_string(),
        AtomicValue::F32(x) => x.to_string(),
        AtomicValue::CUtf8(x) => x.as_ref().to_string(),
        AtomicValue::X(x) => hex::encode(x),
    }
}

pub fn atomic_type<S>(v: &AtomicValue<S>) -> AtomicType {
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

pub fn atomic_schema_serialization_length<S: AsRef<str>>(schema: &indexmap::IndexMap<String, AtomicValue<S>>) -> Result<u16, ()> {
    schema.iter().map(|(_, v)| atomic_value_serialization_length(v)).fold(Ok(0), |a, b| {
        let x = a?;
        let y = b?;
        x.checked_add(y).ok_or(())
    })
}

pub fn atomic_value_serialization_length<S: AsRef<str>>(v: &AtomicValue<S>) -> Result<u16, ()> {
    match v {
        AtomicValue::I8(_) => Ok(1),
        AtomicValue::I16(_) => Ok(2),
        AtomicValue::I32(_) => Ok(4),
        AtomicValue::U8(_) => Ok(1),
        AtomicValue::U16(_) => Ok(2),
        AtomicValue::U32(_) => Ok(4),
        AtomicValue::F32(_) => Ok(4),
        AtomicValue::CUtf8(s) => s.as_ref().bytes().len().try_into().map_err(|_| ()),
        AtomicValue::X(data) => data.len().try_into().map_err(|_| ()),
    }
}

// Serialize, i.e. generate tbl data

#[derive(Debug)]
pub enum SerializeError {
    IOError(std::io::Error),
    TooLong,
}

pub fn serialize_c_utf8<F: std::io::Write>(f: &mut F, s: &str) -> Result<usize, std::io::Error> {
    let mut written = f.write(s.as_bytes())?;
    // TODO: assert that s.as_bytes() doesn't contain 0?
    written += f.write(&[0])?;
    Ok(written)
}

pub fn serialize_atomic_value<S: AsRef<str>, F: std::io::Write>(f: &mut F, v: &AtomicValue<S>) -> Result<usize, std::io::Error> {
    match v {
        AtomicValue::I8(x) => f.write(&x.to_le_bytes()),
        AtomicValue::I16(x) => f.write(&x.to_le_bytes()),
        AtomicValue::I32(x) => f.write(&x.to_le_bytes()),
        AtomicValue::U8(x) => f.write(&x.to_le_bytes()),
        AtomicValue::U16(x) => f.write(&x.to_le_bytes()),
        AtomicValue::U32(x) => f.write(&x.to_le_bytes()),
        AtomicValue::F32(x) => f.write(&x.to_le_bytes()),
        AtomicValue::CUtf8(s) => serialize_c_utf8(f, s.as_ref()),
        AtomicValue::X(data) => f.write(&data),
    }
}

pub fn nf1_schema_serialize<S: AsRef<str>, F: std::io::Write>(f: &mut F, header: &str, schema: &indexmap::IndexMap<String, AtomicValue<S>>) -> Result<usize, SerializeError> {
    let mut written = 0;
    written += serialize_c_utf8(f, header).map_err(SerializeError::IOError)?;
    let length = atomic_schema_serialization_length(schema).map_err(|()| SerializeError::TooLong)?;
    written += f.write(&length.to_le_bytes()).map_err(SerializeError::IOError)?;
    for (_, v) in schema.iter() {
        written += serialize_atomic_value(f, v).map_err(SerializeError::IOError)?;
    }
    Ok(written)
}

// TODO: serialize tbl (version 1 & version 2)

// Deserialize, i.e. read tbl

#[derive(Debug)]
pub enum DeserializeErrorDetail {
    IO(std::io::Error),
    UTF8(std::str::Utf8Error),
    SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total: u16, sum: Option<u32> },
    DuplicateHeader(String),
    EntryHeaderNotInTblHeader { missing: String },
    EntryCountDoesNotMatchHeaderEntry { header: String, expected: u32, actual: usize },
    // TODO:
    //EntryHeaderNotInTblHeader { name: String },
}

#[derive(Clone, Debug)]
pub enum DeserializeErrorPosition {
    Start,
    TblHeader,
    TblHeaderEntry { entry_number: u32 },
    EntryHeader { entry_number: u16 },
    EntryData { entry_number: u16 },
    Attribute(String),
    Type(AtomicType),
    End,
}

#[derive(Debug)]
pub struct DeserializeError {
    detail: DeserializeErrorDetail,
    position: Option<u64>,                            // seek position in the file
    context: std::vec::Vec<DeserializeErrorPosition>, // first element is the position of the original cause, the rest is how we got there; may be empty
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

fn deserialize_c_utf8<F: std::io::Read + std::io::Seek>(f: &mut F, warnings: &mut WarningLogger<DeserializeError>, strict: bool) -> Result<String, DeserializeError> {
    let mut buf = std::vec::Vec::new();
    loop {
        let x = read_byte(f).map_err(|e| DeserializeError {
            detail: DeserializeErrorDetail::IO(e),
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::Type(AtomicType::CUtf8)],
        })?;
        if x != 0 {
            buf.push(x);
        } else {
            break;
        }
    }
    match std::str::from_utf8(buf.as_ref()) {
        Ok(s) => Ok(s.to_string()),
        Err(utf8err) => {
            let s = String::from_utf8_lossy(buf.as_ref());
            let position = (|| {
                let pos: u64 = f.stream_position().ok()?;
                let pos: usize = pos.try_into().ok()?;
                pos.checked_sub(buf.len())?.checked_add(utf8err.valid_up_to() + 1)
            })();
            if strict {
                match position {
                    Some(pos) => log::error!("Invalid UTF-8 String '{}' at position {}: {}", s, pos, utf8err),
                    None => log::error!("Invalid UTF-8 String '{}' at unknown position: {}", s, utf8err),
                }
            } else {
                match position {
                    Some(pos) => log::warn!("Invalid UTF-8 String '{}' at position {}: {}", s, pos, utf8err),
                    None => log::warn!("Invalid UTF-8 String '{}' at unknown position: {}", s, utf8err),
                }
            }
            let e = DeserializeError {
                detail: DeserializeErrorDetail::UTF8(utf8err),
                position: f.stream_position().ok(),
                context: vec![DeserializeErrorPosition::Type(AtomicType::CUtf8)],
            };
            if strict {
                Err(e)
            } else {
                warnings.log(e);
                // TODO: log::trace!(something);
                Ok(s.into_owned())
            }
        }
    }
}

pub fn deserialize_atomic_value<F: std::io::Read + std::io::Seek>(f: &mut F, log: &mut WarningLogger<DeserializeError>, strict: bool, t: &AtomicType) -> Result<AtomicValue<String>, DeserializeError> {
    let wrap_err = |e: std::io::Error, f: &mut F| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::Type(*t)],
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
            let mut buf = vec![0; *n];
            let read = f.read_exact(&mut buf).map_err(|e| wrap_err(e, f))?;
            AtomicValue::X(buf)
        }
    })
}

/// deserialize tbl header for trails of cold steel (no list of headers at the start)
fn deserialize_header_v1<F: std::io::Read + std::io::Seek>(f: &mut F) -> Result<u16, DeserializeError> {
    read_bytes(f).map(|a| u16::from_le_bytes(a)).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })
}

/// deserialize tbl header for trails of cold steel 2 - 5 (includes list of headers at the start)
fn deserialize_header_v2<F: std::io::Read + std::io::Seek>(f: &mut F, log: &mut WarningLogger<DeserializeError>, strict: bool) -> Result<(u16, indexmap::IndexMap<String, u32>), DeserializeError> {
    let number_of_entries = read_bytes(f).map(|a| u16::from_le_bytes(a)).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })?;
    log::trace!("number of entries according to header: {}", number_of_entries);
    let header_counts = deserialize_header_counts(f, log, strict)?;
    let sum_of_counts: Option<u32> = header_counts.values().fold(Some(0), |a, b| a?.checked_add(*b));
    if sum_of_counts != Some(number_of_entries.into()) {
        if strict {
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
    }
    Ok((number_of_entries, header_counts))
}

fn deserialize_header_counts<F: std::io::Read + std::io::Seek>(f: &mut F, log: &mut WarningLogger<DeserializeError>, strict: bool) -> Result<indexmap::IndexMap<String, u32>, DeserializeError> {
    let number_of_headers = u32::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: f.stream_position().ok(),
        context: vec![DeserializeErrorPosition::TblHeader],
    })?);
    log::trace!("number of headers in the file: {}", number_of_headers);

    let mut header_counts = indexmap::IndexMap::new();
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
            indexmap::map::Entry::Vacant(mut e) => {
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
    schema: &indexmap::IndexMap<String, AtomicType>,
) -> Result<indexmap::IndexMap<String, AtomicValue<String>>, DeserializeError> {
    // TODO: use smol_str for index?
    let mut values = indexmap::IndexMap::new();
    for (attr, ty) in schema.iter() {
        let val = deserialize_atomic_value(f, log, strict, ty).map_err(|mut e| {
            e.context.push(DeserializeErrorPosition::Attribute(attr.clone()));
            e
        })?;
        log::trace!("attribute {} = {:?}", attr, val);
        values.insert(attr.clone(), val);
    }
    Ok(values)
}

pub fn deserialize_tbl_v1<F: std::io::Read + std::io::Seek>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schemas: &indexmap::IndexMap<String, indexmap::IndexMap<String, AtomicType>>,
) -> Result<std::vec::Vec<(String, indexmap::IndexMap<String, AtomicValue<String>>)>, DeserializeError> {
    log::debug!("parsing tbl");
    let number_of_entries = deserialize_header_v1(f)?;

    let mut entries = std::vec::Vec::new();

    // parse number of entries equal to tbl header
    for i in 0..number_of_entries {
        let hdr = deserialize_c_utf8(f, log, strict).map_err(|mut e| {
            e.context.push(DeserializeErrorPosition::EntryHeader { entry_number: i });
            e
        })?;
        log::trace!("entry {} header name: {}", i, hdr);
        let len = u16::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
            detail: DeserializeErrorDetail::IO(e),
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::EntryHeader { entry_number: i }],
        })?);
        log::trace!("entry {} data length: {}", i, len);
        // TODO: check if header in header_counts.keys()
        let data = if let Some(schema) = schemas.get(&hdr) {
            log::debug!("parsing data according to schema");
            deserialize_entry_data(f, log, strict, schema).map_err(|mut e| {
                e.context.push(DeserializeErrorPosition::EntryData { entry_number: i });
                e
            })?
        } else {
            log::debug!("parsing data with general parser");
            let x = deserialize_atomic_value(f, log, strict, &AtomicType::X(len.into())).map_err(|mut e| {
                e.context.push(DeserializeErrorPosition::Attribute("data".to_string()));
                e.context.push(DeserializeErrorPosition::EntryData { entry_number: i });
                e
            })?;
            [("data".to_string(), x)].into_iter().collect()
        };
        entries.push((hdr, data));
    }

    // TODO: if strict, check that we're at eof; if not, parse remaining entries
    /*
    let mut i = usize;
    loop {

    }
    */

    Ok(entries)
}

pub fn deserialize_tbl_v2<F: std::io::Read + std::io::Seek>(
    f: &mut F,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schemas: &indexmap::IndexMap<String, indexmap::IndexMap<String, AtomicType>>,
) -> Result<(indexmap::IndexMap<String, u32>, std::vec::Vec<(String, indexmap::IndexMap<String, AtomicValue<String>>)>), DeserializeError> {
    log::debug!("parsing tbl");
    let (number_of_entries, header_counts) = deserialize_header_v2(f, log, strict)?;

    let mut entries = std::vec::Vec::new();

    // parse number of entries equal to tbl header
    for i in 0..number_of_entries {
        let hdr = deserialize_c_utf8(f, log, strict).map_err(|mut e| {
            e.context.push(DeserializeErrorPosition::EntryHeader { entry_number: i });
            e
        })?;
        log::trace!("entry {} header name: {}", i, hdr);
        let len = u16::from_le_bytes(read_bytes(f).map_err(|e| DeserializeError {
            detail: DeserializeErrorDetail::IO(e),
            position: f.stream_position().ok(),
            context: vec![DeserializeErrorPosition::EntryHeader { entry_number: i }],
        })?);
        log::trace!("entry {} data length: {}", i, len);
        // TODO: check if header in header_counts.keys()
        let data = if let Some(schema) = schemas.get(&hdr) {
            log::debug!("parsing data according to schema");
            deserialize_entry_data(f, log, strict, schema).map_err(|mut e| {
                e.context.push(DeserializeErrorPosition::EntryData { entry_number: i });
                e
            })?
        } else {
            log::debug!("parsing data with general parser");
            let x = deserialize_atomic_value(f, log, strict, &AtomicType::X(len.into())).map_err(|mut e| {
                e.context.push(DeserializeErrorPosition::Attribute("data".to_string()));
                e.context.push(DeserializeErrorPosition::EntryData { entry_number: i });
                e
            })?;
            [("data".to_string(), x)].into_iter().collect()
        };
        entries.push((hdr, data));
    }

    // TODO: if strict, check that we're at eof; if not, parse remaining entries
    /*
    let mut i = usize;
    loop {

    }
    */

    // TODO: check header counts

    Ok((header_counts, entries))
}

pub fn load_file_v1<P: AsRef<std::path::Path>>(
    path: P,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schemas: &indexmap::IndexMap<String, indexmap::IndexMap<String, AtomicType>>,
) -> Result<std::vec::Vec<(String, indexmap::IndexMap<String, AtomicValue<String>>)>, DeserializeError> {
    let mut f = std::fs::File::open(path).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: None,
        context: vec![DeserializeErrorPosition::Start],
    })?;
    deserialize_tbl_v1(&mut f, log, strict, schemas)
}

pub fn load_file_v2<P: AsRef<std::path::Path>>(
    path: P,
    log: &mut WarningLogger<DeserializeError>,
    strict: bool,
    schemas: &indexmap::IndexMap<String, indexmap::IndexMap<String, AtomicType>>,
) -> Result<(indexmap::IndexMap<String, u32>, std::vec::Vec<(String, indexmap::IndexMap<String, AtomicValue<String>>)>), DeserializeError> {
    let mut f = std::fs::File::open(path).map_err(|e| DeserializeError {
        detail: DeserializeErrorDetail::IO(e),
        position: None,
        context: vec![DeserializeErrorPosition::Start],
    })?;
    deserialize_tbl_v2(&mut f, log, strict, schemas)
}

// schemas

lazy_static::lazy_static! {
    pub static ref SCHEMAS_CS1 : indexmap::IndexMap<String, indexmap::IndexMap<String, Type>> = [
    ].into_iter().collect();
    pub static ref SCHEMAS_CS1_COMMON : indexmap::IndexMap<String, indexmap::IndexMap<String, Type>> = [
    ].into_iter().collect();

    pub static ref SCHEMAS_CS2 : indexmap::IndexMap<String, indexmap::IndexMap<String, Type>> = [
    ].into_iter().collect();
    pub static ref SCHEMAS_CS2_COMMON : indexmap::IndexMap<String, indexmap::IndexMap<String, Type>> = [
    ].into_iter().collect();

    pub static ref SCHEMAS_CS3 : indexmap::IndexMap<String, indexmap::IndexMap<String, Type>> = [
        ("char_revise".to_string(), [
            ("char".to_string(), Type::CUtf8),
            ("hp".to_string(), Type::U16),
            ("str".to_string(), Type::U16),
            ("def".to_string(), Type::U16),
            ("ats".to_string(), Type::U16),
            ("adf".to_string(), Type::U16),
            ("spd".to_string(), Type::U16),
            ("brk".to_string(), Type::U16),
        ].into_iter().collect())
    ].into_iter().collect();
    pub static ref SCHEMAS_CS3_COMMON : indexmap::IndexMap<String, indexmap::IndexMap<String, Type>> = [
    ].into_iter().collect();


    pub static ref SCHEMAS_CS1_ATOMIC : indexmap::IndexMap<String, indexmap::IndexMap<String, AtomicType>> = get_nf1_for_all(&SCHEMAS_CS1, &SCHEMAS_CS1_COMMON).expect("");
    pub static ref SCHEMAS_CS2_ATOMIC : indexmap::IndexMap<String, indexmap::IndexMap<String, AtomicType>> = get_nf1_for_all(&SCHEMAS_CS2, &SCHEMAS_CS2_COMMON).expect("");
    pub static ref SCHEMAS_CS3_ATOMIC : indexmap::IndexMap<String, indexmap::IndexMap<String, AtomicType>> = get_nf1_for_all(&SCHEMAS_CS3, &SCHEMAS_CS3_COMMON).expect("");
}

// TODO: tests
#[cfg(test)]
mod test {
    #[cfg(feature = "roundtrip-cs3")]
    #[test]
    fn test_roundtrip_cs3() {
        todo!()
    }
}
