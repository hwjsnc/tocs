mod deser;
mod ser;

#[cfg(all(test, feature = "private-tests"))]
mod test;

pub use deser::parse;
pub use ser::serialize;

use crate::io::{PaddedAsciiStr, PaddedAsciiString, PaddedUtf8String};

#[derive(Debug)]
pub struct AlDat {
    unk2: [u8; 4],
    name: PaddedAsciiString<1>,
    name_padding: ScriptNamePadding,
    alignment: std::collections::BTreeMap<PaddedAsciiString<1>, usize>, // TODO: builtin known alignments
    entries: std::vec::Vec<AlDatEntry>,                                 // TODO: BTreeSet? (with name as key)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptNamePadding {
    None,
    Once,
    Twice,
}

#[derive(Debug)]
pub enum AlDatEntry {
    ActionTable(std::vec::Vec<ActionTableEntry>),
    AlgoTable(std::vec::Vec<AlgoTableEntry>),
    Generic { name: PaddedAsciiString<1>, data: std::vec::Vec<u8> },
}

impl AlDatEntry {
    pub fn name_len_with_padding(&self) -> usize {
        match self {
            AlDatEntry::ActionTable { .. } => 12,
            AlDatEntry::AlgoTable { .. } => 10,
            AlDatEntry::Generic { name, .. } => name.len_with_padding().get(),
        }
    }

    pub fn name(&self) -> PaddedAsciiStr<'_, 1> {
        match self {
            // TODO: constify PaddedAsciiStr
            AlDatEntry::ActionTable { .. } => PaddedAsciiStr::new("ActionTable").unwrap(),
            AlDatEntry::AlgoTable { .. } => PaddedAsciiStr::new("AlgoTable").unwrap(),
            AlDatEntry::Generic { name, .. } => name.as_padded_ascii_str(),
        }
    }
}

#[derive(Debug)]
pub struct ActionTableEntry {
    pub id: i16,
    pub unk1: u16,
    pub unk2: [u8; 4],
    pub unk3: f32,
    pub unk4: f32,
    pub unk5: f32,

    pub unk10: [u8; 4],

    pub unk11: u16,
    pub unk12: u16,
    pub unk13: u16,
    pub unk14: u16,
    pub unk15: u16,
    pub unk16: u16,

    pub unk17: [i32; 15],

    pub unk18: u16,
    pub unk19: u16,

    pub flags: PaddedAsciiString<16>,
    pub animation: PaddedUtf8String<32>,
    pub name: PaddedUtf8String<64>,
}

#[derive(Debug)]
pub struct AlgoTableEntry {
    pub action_or_magic_id: i16,
    pub condition: u8,
    pub chance: u8,
    pub use_limit: u8,
    pub target_type: u8,
    pub unk_byte_1: u8,
    pub unk_byte_2: u8,
    pub params: [u32; 6],
}
