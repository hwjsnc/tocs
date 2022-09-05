extern crate blake2;
extern crate ioutil;
extern crate tbl_serialize_derive;
extern crate tbl_serialize_length_derive;

use ioutil::serialize_utf8cstring;
use ioutil::tbl::ParseError;

use ioutil::tbl::Deserialize as _;
use ioutil::tbl::Serialize as _;
use ioutil::tbl::SerializeLength as _;

use blake2::Digest as _;

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
    Item(entry_data::Item),
    QSText(entry_data::QSText),
    Status(entry_data::Status),
}

pub mod entry_data {
    use tbl_deserialize_derive::Deserialize;
    use tbl_serialize_derive::Serialize;
    use tbl_serialize_length_derive::SerializeLength;

    #[derive(Clone, Debug, Serialize, SerializeLength)]
    pub struct Generic {
        #[serialize(ignore)] // header is serialized separately
        pub header: String,
        pub data: Vec<u8>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct Item {
        pub id: u16,
        pub unknown_byte_1: u8,
        pub unknown_byte_2: u8,
        pub flags: String,
        #[deserialize(length = 46)]
        pub unknown_data: std::vec::Vec<u8>,
        pub name: String,
        pub description: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSText {
        pub unknown_short: u16,
        pub unknown_byte_1: u8,
        pub text: String,
        pub unknown_byte_2: u8,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct Status {
        pub unknown_text_1: String,
        pub unknown_text_2: String,
        pub unknown_text_3: String,
        #[deserialize(length = 112)]
        pub unknown_data: std::vec::Vec<u8>,
        pub flags: String,
        pub name: String,
        pub description: String,
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
            Self::Item(_) => "item",
            Self::QSText(_) => "QSText",
            Self::Status(_) => "status",
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
        static ref PARSERS: std::collections::HashMap<&'static str, ParseFun> = [
            ("item", (|ctx| entry_data::Item::deserialize(ctx).map(|(data, ctx)| (Entry::Item(data), ctx))) as ParseFun),
            ("QSText", (|ctx| entry_data::QSText::deserialize(ctx).map(|(data, ctx)| (Entry::QSText(data), ctx))) as ParseFun),
            ("status", (|ctx| entry_data::Status::deserialize(ctx).map(|(data, ctx)| (Entry::Status(data), ctx))) as ParseFun),
        ].into_iter().collect();
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

pub fn fix(data: &[u8]) -> std::vec::Vec<u8> {
    let mut copy = data.to_vec();
    let _ = fix_in_place_if_necessary(&mut copy);
    copy
}

pub fn fix_in_place_if_necessary(data: &mut std::vec::Vec<u8>) -> Option<()> {
    type FixFun = fn(&mut std::vec::Vec<u8>) -> ();
    lazy_static::lazy_static! {
        static ref FIXFUN: std::collections::HashMap<[u8; 64], FixFun> = [
                (hex_literal::hex!("e3a75ea6c0902da1f8da0d052305a4db98839a8a3c4403cf129144177a3910504a4c8b11ce8aa177db3a7554c58575e86a41655fb8414a90ce2c3083ecef88d5"), fix_CS1_PC_1_6_dat_us_t_main as FixFun),
        ].into_iter().collect();
    }
    let hash = blake2::Blake2b512::digest(&data);
    let fixfun = FIXFUN.get(&hash[..])?;
    fixfun(data);
    Some(())
}

fn replace_byte(data: &mut [u8], location: usize, expected: u8, replacement: u8) {
    assert_eq!(data[location], expected);
    data[location] = replacement;
}

#[allow(non_snake_case)]
fn fix_CS1_PC_1_6_dat_us_t_main(data: &mut std::vec::Vec<u8>) {
    // Cold Steel PC version 1.6, data/text/dat_us/t_main.tbl
    assert_eq!(data.len(), 155086);
    let replace = [(127310, 0x54, 0x47), (128670, 0xa4, 0x53)];
    for (location, expected, replacement) in replace {
        replace_byte(data, location, expected, replacement);
    }
}

#[cfg(test)]
mod test {
    use super::{fix, load, store};
    extern crate walkdir;

    #[test]
    #[cfg(feature = "roundtrip-cs1")]
    fn test_roundtrip() {
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs1/data/text/"));
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(parent) = path.parent() {
                if parent.file_name() == Some(&std::ffi::OsStr::new("dat")) {
                    continue; // TODO: support for JP files, maybe -- they use shift-JIS encoding for text though ...
                }
            }
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                println!("{}", path.display());
                let original = std::fs::read(path).unwrap();
                let fixed = fix(&original);
                let tbl = load(&fixed).unwrap();
                let serialized = store(&tbl).unwrap();
                assert_eq!(serialized, fixed);
            }
        }
    }
}
