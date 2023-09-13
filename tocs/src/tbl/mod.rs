mod deser;
mod schema;
pub mod schemas;
mod ser;
pub mod value_conversion;

pub mod fixes;

use smallvec::SmallVec;

pub use schema::{get_nf1_for_all, AtomicAttributeValues, AtomicSchema, AtomicSchemas, AtomicType, AtomicValue, Attribute, Header, Schema, Schemas, Type, Value};
pub use ser::{serialize_tbl_v1, serialize_tbl_v2, SerializeError};
pub use value_conversion::try_atomic_value_from_str;

// tests
#[cfg(all(test, feature = "private-tests"))]
mod test;

pub use deser::{DeserializeError, DeserializeErrorDetail, DeserializeErrorPosition};

#[derive(Debug)]
pub enum ReadError {
    Io(std::io::Error),
    Parse(deser::DeserializeError),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReadError::Io(err) => write!(f, "couldn't read file: {err}"),
            ReadError::Parse(err) => write!(f, "couldn't parse file: {err}"),
        }
    }
}

impl std::error::Error for ReadError {}

// generic load/fix/store

#[derive(Clone, Copy, Debug)]
enum VersionWithoutSchema {
    V1, // no header counts: CS1
    V2, // with header counts: CS2 onwards
}

#[derive(Clone, Debug)]
pub enum Version {
    CS1En,
    CS2En,
    CS3,
    CS4,
    CS5,
    V1(AtomicSchemas), // generic version with custom schema (no header counts / CS1 version)
    V2(AtomicSchemas), // generic version with custom schema (with header counts / CS2 onward)
}

impl std::str::FromStr for Version {
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
            "cs5" => Ok(Version::CS5),
            "CS5" => Ok(Version::CS5),
            "reverie" => Ok(Version::CS5),
            "Reverie" => Ok(Version::CS5),
            "v1" => Ok(Version::V1(schemas::EMPTY.clone())),
            "v2" => Ok(Version::V2(schemas::EMPTY.clone())),
            _ => Err("invalid/unknown tbl version"),
        }
    }
}

impl Version {
    fn without_schema(&self) -> VersionWithoutSchema {
        match self {
            Version::CS1En => VersionWithoutSchema::V1,
            Version::CS2En => VersionWithoutSchema::V2,
            Version::CS3 => VersionWithoutSchema::V2,
            Version::CS4 => VersionWithoutSchema::V2,
            Version::CS5 => VersionWithoutSchema::V2,
            Version::V1(_) => VersionWithoutSchema::V1,
            Version::V2(_) => VersionWithoutSchema::V2,
        }
    }

    pub fn schemas(&self) -> &AtomicSchemas {
        match self {
            Version::CS1En => &schemas::CS1_ATOMIC,
            Version::CS2En => &schemas::CS2_ATOMIC,
            Version::CS3 => &schemas::CS3_ATOMIC,
            Version::CS4 => &schemas::CS4_ATOMIC,
            Version::CS5 => &schemas::CS5_ATOMIC,
            Version::V1(schema) => schema,
            Version::V2(schema) => schema,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tbl {
    version: VersionWithoutSchema,
    headers: SmallVec<[Header; 6]>, // TODO: just make it a vec ffs
    entries: std::vec::Vec<Entry>,
}

impl Tbl {
    pub fn new<I: IntoIterator<Item = Header>, J: IntoIterator<Item = Entry>>(version: &Version, headers: I, entries: J) -> Self {
        Self {
            version: version.without_schema(),
            headers: headers.into_iter().collect(),
            entries: entries.into_iter().collect(),
        }
    }

    fn new_2(version: VersionWithoutSchema, headers: SmallVec<[Header; 6]>, entries: std::vec::Vec<Entry>) -> Self {
        Self { version, headers, entries }
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
    KnownFixApplied(Tbl),
    Warn { tbl: Tbl, warnings: std::vec::Vec<DeserializeError> },
    Err(ReadError),
}

pub enum ParseTblResult {
    Ok(Tbl),
    Warn { tbl: Tbl, warnings: std::vec::Vec<DeserializeError> },
    Err(DeserializeError),
}

impl ReadTblResult {
    pub fn ignore_warnings(self) -> Result<Tbl, ReadError> {
        match self {
            ReadTblResult::Ok(tbl) => Ok(tbl),
            ReadTblResult::KnownFixApplied(tbl) => Ok(tbl),
            ReadTblResult::Warn { tbl, .. } => Ok(tbl),
            ReadTblResult::Err(e) => Err(e),
        }
    }
}

impl From<ParseTblResult> for ReadTblResult {
    fn from(result: ParseTblResult) -> ReadTblResult {
        match result {
            ParseTblResult::Ok(tbl) => ReadTblResult::Ok(tbl),
            ParseTblResult::Warn { tbl, warnings } => ReadTblResult::Warn { tbl, warnings },
            ParseTblResult::Err(err) => ReadTblResult::Err(ReadError::Parse(err)),
        }
    }
}

fn warnings_are_likely_fine(tbl: &Tbl, schemas: &AtomicSchemas, warnings: &[DeserializeError]) -> bool {
    for w in warnings.iter() {
        println!("WARNING: {w}");
    }

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
                    let header: &Header = &entries[number].header;
                    if !schemas.contains_key(header) {
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
                DeserializeErrorDetail::Eof => return false,
                DeserializeErrorDetail::Ascii => return false,
                DeserializeErrorDetail::Utf8(_, _) => return false,
                DeserializeErrorDetail::SumOfEntryNumbersDiffersFromTotalEntryNumber { .. } => {} // it's probably fine
                DeserializeErrorDetail::DuplicateHeader(_) => {}
                DeserializeErrorDetail::EntryHeaderNotInTblHeader { .. } => {}
                DeserializeErrorDetail::EntryCountDoesNotMatchHeaderEntry { header, .. } => {
                    if schemas.contains_key(header) {
                        // should be fine
                    } else {
                        (); //return false; TODO: add missing schemas that make this fail tests
                    }
                }
                DeserializeErrorDetail::EntryDataLengthMismatch { .. } => {
                    return false;
                }
                DeserializeErrorDetail::UnexpectedNumberOfEntries { .. } => {} // it's probably fine
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

pub fn read<F: std::io::Read>(f: &mut F, version: &Version) -> ReadTblResult {
    let mut data = std::vec::Vec::new();
    match f.read_to_end(&mut data) {
        Ok(_) => parse(&data, version).into(),
        Err(err) => ReadTblResult::Err(ReadError::Io(err)),
    }
}

pub fn write<F: std::io::Write>(f: &mut F, tbl: &Tbl) -> Result<usize, SerializeError> {
    let written = match tbl.version {
        VersionWithoutSchema::V1 => serialize_tbl_v1(f, &tbl.entries)?,
        VersionWithoutSchema::V2 => serialize_tbl_v2(f, &tbl.headers, &tbl.entries)?,
    };
    f.flush().map_err(SerializeError::IoError)?;
    Ok(written)
}

pub fn parse_once(data: &[u8], version: &Version) -> Result<Tbl, DeserializeError> {
    deser::TblReaderAtomic::new(data, version).read()
}

pub fn parse_once_lax(data: &[u8], version: &Version) -> (Result<Tbl, DeserializeError>, std::vec::Vec<DeserializeError>) {
    let mut warnings = std::vec::Vec::new();
    let result = deser::TblReaderAtomic::new_lax(data, version, &mut warnings).read();
    (result, warnings)
}

// TODO: parse (&[u8]) + read (impl std::io::Read)
// distinct error types: parse cannot have io errors

pub fn parse(data: &[u8], version: &Version) -> ParseTblResult {
    match parse_once(data, version) {
        Ok(tbl) => return ParseTblResult::Ok(tbl),
        Err(first_error) => match parse_once_lax(data, version) {
            (Ok(tbl), warnings) if warnings_are_likely_fine(&tbl, version.schemas(), &warnings) => ParseTblResult::Warn { tbl, warnings: warnings },
            _ => ParseTblResult::Err(first_error),
        },
    }
}

/// Read a tbl file into memory, apply known fixes, and attempt to parse it.
pub fn read_file_with_fixes<P: AsRef<std::path::Path>>(path: P, version: &Version) -> ReadTblResult {
    let mut data = match std::fs::read(path) {
        Ok(data) => data,
        Err(err) => {
            return ReadTblResult::Err(ReadError::Io(err));
        }
    };
    if fixes::fix_in_place(version, &mut data).was_fixed() {
        let tbl = parse_once(data.as_slice(), version).expect("we applied fixes so parsing should succeed");
        ReadTblResult::KnownFixApplied(tbl)
    } else {
        parse(data.as_slice(), version).into()
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
