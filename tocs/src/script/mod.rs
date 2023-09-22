mod deser;
mod ser;

#[cfg(all(test, feature = "private-tests"))]
mod test;

pub use deser::parse;
pub use ser::serialize;

use crate::io::{PaddedAsciiStr, PaddedAsciiString, PaddedUtf8String};

#[derive(Debug)]
pub struct Script {
    pub name: PaddedAsciiString<1>,
    pub name_padding: ScriptNamePadding,
    pub alignment: std::collections::BTreeMap<PaddedAsciiString<1>, usize>, // TODO: builtin known alignments
    pub entries: std::vec::Vec<ScriptEntry>,                                // TODO: BTreeSet? (with name as key)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptNamePadding {
    None,
    Once,
    Twice,
}

#[derive(Debug)]
pub enum ScriptEntry {
    ActionTable(std::vec::Vec<ActionTableEntry>),
    AlgoTable(std::vec::Vec<AlgoTableEntry>),
    Generic { name: PaddedAsciiString<1>, data: std::vec::Vec<u8> },
}

impl ScriptEntry {
    pub fn name_len_with_padding(&self) -> usize {
        match self {
            ScriptEntry::ActionTable { .. } => 12,
            ScriptEntry::AlgoTable { .. } => 10,
            ScriptEntry::Generic { name, .. } => name.len_with_padding().get(),
        }
    }

    pub fn name(&self) -> PaddedAsciiStr<'_, 1> {
        match self {
            // TODO: constify PaddedAsciiStr
            ScriptEntry::ActionTable { .. } => PaddedAsciiStr::new("ActionTable").unwrap(),
            ScriptEntry::AlgoTable { .. } => PaddedAsciiStr::new("AlgoTable").unwrap(),
            ScriptEntry::Generic { name, .. } => name.as_padded_ascii_str(),
        }
    }
}

#[derive(Debug)]
pub struct ActionTableEntry {
    pub id: i16,
    pub category: u16,
    pub target_type: u8,
    pub target_range: u8,
    pub target_size: u16,

    pub unk4: f32,
    pub unk5: f32,
    pub unk6: f32,

    pub cast_delay: u16,
    pub recovery_delay: u16,

    pub effects: [Effect; 5],
    pub unk10: u16,

    pub unk18: u32, // cost?

    pub flags: PaddedAsciiString<16>,
    pub animation: PaddedUtf8String<32>,
    pub name: PaddedUtf8String<64>,
}

#[derive(Debug, Clone, Copy)]
pub struct Effect {
    pub id: u16,
    pub parameters: [i32; 3],
}

#[derive(Debug)]
pub struct AlgoTableEntry {
    pub action_or_magic_id: i16,
    pub condition: u8,
    pub chance: u8,
    pub use_limit: u8,
    pub target_priority: u8,
    pub unk_byte_1: u8,
    pub unk_byte_2: u8,
    pub cond_param_1: u32,
    pub cond_param_2: u32,
    pub cond_param_3: u32,
    pub target_param: u32,
    pub use_limit_after_retry: u32,
    pub unk6: u32,
}
