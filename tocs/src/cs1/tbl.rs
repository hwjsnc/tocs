extern crate ioutil;
use ioutil::parse_eof;
use ioutil::serialize_utf8cstring;
use ioutil::tbl::{ParseError, ParseErrorContext, ParseErrorDetail};

use std::vec::Vec;

#[derive(Clone, Debug)]
pub struct Tbl {
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
pub enum Entry {
    Generic(entry_data::Generic),
}

pub mod entry_data {
    #[derive(Clone, Debug)]
    pub struct Generic {
        pub header: String,
        pub data: Vec<u8>,
    }
}

impl Entry {
    pub fn header_name(&self) -> &str {
        match self {
            Self::Generic(entry_data::Generic { header, .. }) => header,
        }
    }
}

impl ioutil::tbl::TblEntry for Entry {
    fn new_generic(header_name: &str, data: &[u8]) -> Self {
        Entry::Generic(entry_data::Generic {
            header: header_name.to_string(),
            data: data.to_vec(),
        })
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

    let (number_of_entries, ctx) = ioutil::tbl::parse_tbl_header_v1(ioutil::ParseContext::start_of(input))?;
    let (entries, ctx) = ioutil::tbl::parse_entries(&PARSERS, ctx, number_of_entries)?;

    if ctx.strict {
        let _ = parse_eof(ctx).map_err(|e| ParseError {
            detail: ParseErrorDetail::Primitive(e),
            context: ParseErrorContext::End,
        })?;
    }

    Ok(Tbl { entries })
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
        &serialized_data_length(entry)
            .ok_or_else(|| SerializeError::EntryTooLong {
                header: entry.header_name().to_string(),
            })?
            .to_le_bytes(),
    )
    .map_err(SerializeError::WriteError)?;
    match entry {
        Entry::Generic(entry_data::Generic { data, .. }) => {
            out.write(data).map_err(SerializeError::WriteError)?;
        }
    }
    Ok(())
}

fn serialized_data_length(entry: &Entry) -> Option<u16> {
    match entry {
        Entry::Generic(entry_data::Generic { data, .. }) => data.len().try_into().ok(),
    }
}

#[cfg(test)]
mod test {
    use super::load;
    extern crate walkdir;

    #[test]
    fn test() {
        todo!("test each individual table header type?");
    }

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
                let _tbl = load(&original)/*.unwrap()*/;
                /*
                let serialized = store(&tbl).unwrap();
                assert_eq!(original, serialized);
                */
            }
        }
    }
}
