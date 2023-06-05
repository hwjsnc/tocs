use indexmap::IndexMap;
use smallvec::SmallVec;
use smol_str::SmolStr;

pub type SmallBuf = SmallVec<[u8; 16]>;

// Schema definition types

/// Types as they appear in the schema definition
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
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
    Ref(Header),
}

/// Values corresponding to the types available in the schema
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

/// Atomic types; after unrolling repetitions and dereferencing references -- for converting to/from CSV/SQL
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

/// Atomic values - values corresponding to the atomic types
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

pub struct Schemas {
    pub entries: IndexMap<Header, Schema>,
    pub common: IndexMap<Header, Schema>,
}

// useful type definitions
pub type Header = crate::io::PaddedAsciiString<1>;
pub type Attribute = SmolStr; // attributes do not need to be serialized or deserialized

pub type Schema = IndexMap<Attribute, Type>;
pub type AttributeValues = IndexMap<Attribute, Value>;

pub type AtomicSchema = IndexMap<Attribute, AtomicType>;
pub type AtomicSchemas = IndexMap<Header, AtomicSchema>;
pub type AtomicAttributeValues = IndexMap<Attribute, AtomicValue>;

// Convert schema Type to Atomic Type (by unrolling sequences and dereferencing common structures such as effects)

#[derive(Clone, Debug)]
pub enum NF1ErrorDetail {
    NoSuchSchema(Header),
    Duplicate(Header),
}

#[derive(Clone, Debug)]
pub enum NF1ErrorPosition {
    SchemaLookup,
    Deref(Header),
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

#[cfg(feature = "serde")]
impl serde::Serialize for Type {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStructVariant as _;

        match self {
            Type::I8 => ser.serialize_unit_variant("type", 0, "i8"),
            Type::I16 => ser.serialize_unit_variant("type", 1, "i16"),
            Type::I32 => ser.serialize_unit_variant("type", 2, "i32"),
            Type::U8 => ser.serialize_unit_variant("type", 3, "u8"),
            Type::U16 => ser.serialize_unit_variant("type", 4, "u16"),
            Type::U32 => ser.serialize_unit_variant("type", 5, "u32"),
            Type::F32 => ser.serialize_unit_variant("type", 6, "f32"),
            Type::CUtf8 => ser.serialize_unit_variant("type", 7, "c_utf8"),
            Type::X(n) => {
                let mut tv = ser.serialize_struct_variant("type", 8, "bytes", 1)?;
                tv.serialize_field("length", n)?;
                tv.end()
            }
            Type::Repeat(n, ty) => {
                let mut sv = ser.serialize_struct_variant("type", 9, "repeat", 2)?;
                sv.serialize_field("count", n)?;
                sv.serialize_field("type", ty)?;
                sv.end()
            }
            Type::Ref(hdr) => {
                let mut tv = ser.serialize_struct_variant("type", 10, "ref", 1)?;
                tv.serialize_field("header", hdr.as_str())?;
                tv.end()
            }
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Schemas {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeMap, SerializeStruct};
        struct SchemaSerializer<'a>(&'a indexmap::IndexMap<Header, indexmap::IndexMap<Attribute, Type>>);
        impl<'a> serde::Serialize for SchemaSerializer<'a> {
            fn serialize<S2: serde::Serializer>(&self, ser: S2) -> Result<S2::Ok, S2::Error> {
                let mut map = ser.serialize_map(Some(self.0.len()))?;
                for (hdr, schema) in self.0.iter() {
                    map.serialize_key(hdr.as_str())?;
                    map.serialize_value(schema)?;
                }
                map.end()
            }
        }

        let mut s = ser.serialize_struct("schemas", 2)?;
        s.serialize_field("entries", &SchemaSerializer(&self.entries))?;
        s.serialize_field("common", &SchemaSerializer(&self.common))?;
        s.end()
    }
}

impl Type {
    pub fn try_to_atomic_type(&self) -> Option<AtomicType> {
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

impl AtomicType {
    pub fn describe(&self) -> &'static str {
        match self {
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

    // for creating sqlite tbl schemas
    #[cfg(feature = "rusqlite")]
    pub fn sqlite_type(&self) -> &'static str {
        match self {
            AtomicType::I8 | AtomicType::I16 | AtomicType::I32 | AtomicType::U8 | AtomicType::U16 | AtomicType::U32 => "int",
            AtomicType::F32 => "real",
            AtomicType::CUtf8 => "text",
            AtomicType::X(_) => "blob",
        }
    }
}

impl AtomicValue {
    pub fn r#type(&self) -> AtomicType {
        match self {
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
}

// "Flatten" the schema by unrolling sequences and dereferencing references
pub fn get_nf1(schema_name: &Header, schemas: &indexmap::IndexMap<Header, Schema>) -> Result<AtomicSchema, NF1Error> {
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
                        nf1.insert(SmolStr::new(format!("{}[{}]", name, i)), t);
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

pub fn get_nf1_for_all(schemas: &Schemas) -> Result<AtomicSchemas, NF1Error> {
    let mut joint = schemas.entries.clone();
    joint.extend(schemas.common.clone().into_iter());
    let joint = joint;
    let mut res = IndexMap::new();
    for name in schemas.entries.keys() {
        res.insert(name.clone(), get_nf1(name, &joint)?);
    }
    Ok(res)
}
