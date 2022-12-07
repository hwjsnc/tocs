// Conversion of schema values to/from other types

use smol_str::SmolStr;

use hex::FromHex as _;
use std::str::FromStr as _;

#[cfg(feature = "rusqlite")]
use rusqlite::types::FromSql as _;

use super::schema::{AtomicType, AtomicValue};

pub enum AtomicValueFromStringError {
    Generic,
    LengthMismatch { expected: usize, actual: usize },
}

// conversion to/from strings

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

impl AtomicValue {
    pub fn from_str(ty: &AtomicType, text: &str) -> Result<AtomicValue, AtomicValueFromStringError> {
        try_atomic_value_from_str(ty, text)
    }
}

impl std::fmt::Display for AtomicValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            AtomicValue::I8(x) => write!(f, "{}", x),
            AtomicValue::I16(x) => write!(f, "{}", x),
            AtomicValue::I32(x) => write!(f, "{}", x),
            AtomicValue::U8(x) => write!(f, "{}", x),
            AtomicValue::U16(x) => write!(f, "{}", x),
            AtomicValue::U32(x) => write!(f, "{}", x),
            AtomicValue::F32(x) => write!(f, "{}", x),
            AtomicValue::CUtf8(x) => write!(f, "{}", x),
            AtomicValue::X(x) => write!(f, "{}", hex::encode(x)), // TODO: shouldn't need to allocate here, but https://github.com/KokaKiwi/rust-hex/issues/8
        }
    }
}

// conversion to/from sqlite

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
                // This conversion is slower than "as" conversion, but no less accurate.
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
            AtomicType::X(_) => val.as_blob().map(|blob| AtomicValue::X(blob.into())),
        }
    }
}
