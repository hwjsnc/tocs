extern crate ioutil;
extern crate tbl_serialize_derive;
extern crate tbl_serialize_length_derive;

use ioutil::parse_eof;
use ioutil::serialize_utf8cstring;
use ioutil::tbl::{ParseError, ParseErrorContext, ParseErrorDetail};

use ioutil::tbl::Deserialize as _;
use ioutil::tbl::Serialize as _;
use ioutil::tbl::SerializeLength as _;

pub use ioutil::tbl::TblEntry;

use tbl_serialize_derive::Serialize;
use tbl_serialize_length_derive::SerializeLength;

use std::vec::Vec;

#[derive(Clone, Debug)]
pub struct Tbl {
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, Serialize, SerializeLength)]
pub enum Entry {
    Generic(entry_data::Generic),
}

pub mod entry_data {
    use tbl_serialize_derive::Serialize;
    use tbl_serialize_length_derive::SerializeLength;

    #[derive(Clone, Debug, Serialize, SerializeLength)]
    pub struct Generic {
        #[serialize(ignore)] // header is serialized separately
        pub header: String,
        pub data: Vec<u8>,
    }
}

impl ioutil::tbl::Tbl<Entry> for Tbl {
    fn new(_headers: std::vec::Vec<String>, entries: std::vec::Vec<Entry>) -> Self {
        Tbl { entries }
    }
}

impl ioutil::tbl::TblEntry for Entry {
    fn new_generic(header_name: &str, data: &[u8]) -> Self {
        Entry::Generic(entry_data::Generic {
            header: header_name.to_string(),
            data: data.to_vec(),
        })
    }
    fn header_name(&self) -> &str {
        match self {
            Self::Generic(entry_data::Generic { header, .. }) => header,
        }
    }
}

#[derive(Debug)]
pub enum SerializeError {
    // TODO: number of offending entry
    TooMany { which: String, max: usize },
    EntryTooLong { header: String },
    UnexpectedEntryLength { header: String, length_prefix: u16, parsed_length: Option<u16> },
    WriteError(std::io::Error),
}

pub fn load(input: &[u8]) -> Result<Tbl, ParseError> {
    type ParseFun = ioutil::tbl::EntryParseFun<Entry>;
    lazy_static::lazy_static! {
        static ref PARSERS: std::collections::HashMap<&'static str, ParseFun> = [].into_iter().collect();
    }
    let parse_options = ioutil::tbl::ParseOptions {
        with_header_counts: false,
        strict: true,
    };
    ioutil::tbl::parse_tbl::<Entry, Tbl>(&PARSERS, parse_options, input)
}

pub fn store(tbl: &Tbl) -> Result<std::vec::Vec<u8>, SerializeError> {
    let mut data: std::vec::Vec<u8> = std::vec::Vec::new();
    let number_of_entries: u16 = tbl.entries.len().try_into().map_err(|_| SerializeError::TooMany {
        which: "total number of entries".to_string(),
        max: std::u16::MAX as usize,
    })?;
    data.extend_from_slice(&number_of_entries.to_le_bytes());
    for entry in tbl.entries.iter() {
        serialize_entry(&mut data, entry)?;
    }
    Ok(data)
}

fn serialize_entry<W>(out: &mut W, entry: &Entry) -> Result<(), SerializeError>
where
    W: std::io::Write,
{
    // header name and size
    serialize_utf8cstring(out, entry.header_name()).map_err(SerializeError::WriteError)?;
    out.write(
        &entry
            .data_length()
            .ok_or_else(|| SerializeError::EntryTooLong {
                header: entry.header_name().to_string(),
            })?
            .to_le_bytes(),
    )
    .map_err(SerializeError::WriteError)?;
    entry.serialize(out).map_err(SerializeError::WriteError)
}

#[cfg(test)]
mod test {
    use super::{load, store};
    extern crate walkdir;

    #[test]
    #[cfg(feature = "roundtrip-cs1")]
    fn test_roundtrip() {
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs1/data/text/"));
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                println!("{}", path.display());
                let original = std::fs::read(path).unwrap();
                // TODO: fix data files
                let tbl = load(&original).unwrap();
                let serialized = store(&tbl).unwrap();
                assert_eq!(original, serialized);
            }
        }
    }
}
