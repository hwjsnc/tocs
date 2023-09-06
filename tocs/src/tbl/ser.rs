// tbl serialization: create .tbl file from in-memory representation

use super::schema::{AtomicAttributeValues, AtomicValue, Header};
use super::Entry;

pub fn atomic_attributes_serialization_length(schema: &AtomicAttributeValues) -> Option<u16> {
    // note: we're not using the serialization length of the schema because it can vary (if strings or opaque blobs are present)
    schema.iter().map(|(_, v)| atomic_value_serialization_length(v)).try_fold(0u16, |a, b| a.checked_add(b?))
}

pub fn atomic_value_serialization_length(v: &AtomicValue) -> Option<u16> {
    match v {
        AtomicValue::I8(_) => Some(1),
        AtomicValue::I16(_) => Some(2),
        AtomicValue::I32(_) => Some(4),
        AtomicValue::U8(_) => Some(1),
        AtomicValue::U16(_) => Some(2),
        AtomicValue::U32(_) => Some(4),
        AtomicValue::F32(_) => Some(4),
        AtomicValue::CUtf8(s) => (s.len() + 1).try_into().ok(),
        AtomicValue::X(data) => data.len().try_into().ok(),
    }
}

#[derive(Debug)]
pub enum SerializeError {
    IoError(std::io::Error),
    TooManyHeaders,
    TooManyEntries,
    EntryDataTooLong,
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SerializeError::IoError(e) => write!(f, "{}", e),
            SerializeError::TooManyHeaders => write!(f, "too many headers in tbl (max 4294967295)"),
            SerializeError::TooManyEntries => write!(f, "too many entries in tbl (max 65535)"),
            SerializeError::EntryDataTooLong => write!(f, "entry data too long (max 65535bytes))"),
        }
    }
}

impl std::error::Error for SerializeError {}

impl AtomicValue {
    fn write<F: std::io::Write>(&self, f: &mut F) -> Result<usize, std::io::Error> {
        match self {
            AtomicValue::I8(x) => f.write_all(&x.to_le_bytes()).map(|()| 1),
            AtomicValue::I16(x) => f.write_all(&x.to_le_bytes()).map(|()| 2),
            AtomicValue::I32(x) => f.write_all(&x.to_le_bytes()).map(|()| 4),
            AtomicValue::U8(x) => f.write_all(&x.to_le_bytes()).map(|()| 1),
            AtomicValue::U16(x) => f.write_all(&x.to_le_bytes()).map(|()| 2),
            AtomicValue::U32(x) => f.write_all(&x.to_le_bytes()).map(|()| 4),
            AtomicValue::F32(x) => f.write_all(&x.to_le_bytes()).map(|()| 4),
            AtomicValue::CUtf8(s) => {
                f.write_all(s.as_bytes())?;
                f.write_all(&[0])?;
                Ok(1 + s.len())
            }
            AtomicValue::X(data) => f.write_all(data).map(|()| data.len()),
        }
    }
}

pub fn nf1_schema_serialize<F: std::io::Write>(f: &mut F, header: &Header, attributes: &AtomicAttributeValues) -> Result<usize, SerializeError> {
    let mut written = 0;
    written += header.write_with_terminator(f).map_err(SerializeError::IoError)?;
    let length = atomic_attributes_serialization_length(attributes).ok_or(SerializeError::EntryDataTooLong)?;
    written += f.write(&length.to_le_bytes()).map_err(SerializeError::IoError)?;
    for (_, v) in attributes.iter() {
        written += v.write(f).map_err(SerializeError::IoError)?;
    }
    Ok(written)
}

pub fn serialize_tbl_v1<F: std::io::Write>(f: &mut F, entries: &[Entry]) -> Result<usize, SerializeError> {
    let mut written = 0;
    let number_of_entries: Result<u16, _> = entries.len().try_into();
    match number_of_entries {
        Ok(n) => {
            written += f.write(&n.to_le_bytes()).map_err(SerializeError::IoError)?;
        }
        Err(_) => {
            log::error!("too many entries in file ({})", entries.len());
            return Err(SerializeError::TooManyEntries);
        }
    }
    for entry in entries {
        written += nf1_schema_serialize(f, &entry.header, &entry.values)?;
    }
    Ok(written)
}

pub fn serialize_tbl_v2<F: std::io::Write>(f: &mut F, headers: &[Header], entries: &[Entry]) -> Result<usize, SerializeError> {
    let mut written = 0;
    let number_of_entries: Result<u16, _> = entries.len().try_into();
    match number_of_entries {
        Ok(n) => {
            written += f.write(&n.to_le_bytes()).map_err(SerializeError::IoError)?;
        }
        Err(_) => {
            log::error!("too many entries in file ({})", entries.len());
            return Err(SerializeError::TooManyEntries);
        }
    }
    let number_of_headers: u32 = headers.len().try_into().map_err(|_| SerializeError::TooManyHeaders)?;
    written += f.write(&number_of_headers.to_le_bytes()).map_err(SerializeError::IoError)?;
    for header in headers.iter() {
        let n: u32 = entries.iter().filter(|e| &e.header == header).count().try_into().expect("total number of entries fits in a u16");
        written += header.write_with_terminator(f).map_err(SerializeError::IoError)?;
        written += f.write(&n.to_le_bytes()).map_err(SerializeError::IoError)?;
    }
    for entry in entries {
        written += nf1_schema_serialize(f, &entry.header, &entry.values)?;
    }
    Ok(written)
}
