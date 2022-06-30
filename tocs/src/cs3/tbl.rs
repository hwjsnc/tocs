extern crate blake2;
extern crate hex_literal;
extern crate ioutil;
extern crate lazy_static;
extern crate tbl_deserialize_derive;
extern crate tbl_serialize_derive;
extern crate tbl_serialize_length_derive;

use ioutil::tbl::{ParseError, ParseErrorContext, ParseErrorDetail};
use ioutil::{parse_eof, ParseContext};

use ioutil::tbl::Deserialize as _;
use ioutil::tbl::Serialize as _;
use ioutil::tbl::SerializeLength as _;

use blake2::Digest as _;

use std::vec::Vec;

use tbl_serialize_derive::Serialize;
use tbl_serialize_length_derive::SerializeLength;

#[derive(Clone, Debug)]
pub struct Tbl {
    pub headers: Vec<String>,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, Serialize, SerializeLength)]
pub enum Entry {
    Generic(entry_data::Generic),
    ActiveVoiceTableData(entry_data::ActiveVoiceTableData),
    AtBonus(entry_data::AtBonus),
    CharRevise(entry_data::CharRevise),
    CharData(entry_data::CharData),
    CompHelpData(entry_data::CompHelpData),
    GameDifficulty(entry_data::GameDifficulty),
    Item(entry_data::Item),
    Magic(entry_data::Magic),
    MagicBo(entry_data::MagicBo),
    MasterQuartzDummy(entry_data::MasterQuartzDummy),
    SE(entry_data::SE),
    SlotCost(entry_data::SlotCost),
    Status(entry_data::Status),
    StatusP(entry_data::StatusP),
    Voice(entry_data::Voice),
}

pub mod entry_data {
    #[cfg(feature = "sqliteio")]
    extern crate sqliteio_derive;
    #[cfg(feature = "sqliteio")]
    use sqliteio_derive::SqliteIO;

    use tbl_deserialize_derive::Deserialize;
    use tbl_serialize_derive::Serialize;
    use tbl_serialize_length_derive::SerializeLength;

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "generic"))]
    #[derive(Clone, Debug, Serialize, SerializeLength)]
    pub struct Generic {
        #[serialize(ignore)] // we're serializing the header separately
        pub header: String,
        pub data: Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "ActiveVoiceTableData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct ActiveVoiceTableData {
        pub unknown_short_1: u16,
        pub unknown_short_2: u16,
        pub unknown_short_3: u16,
        pub portrait: String,
        pub id: u16,
        pub character: i16,
        pub text: String,
        pub unknown_float_1: f32,
        pub unknown_float_2: f32,
        #[deserialize(length = 12)]
        pub unknown_data_2: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "AtBonus"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct AtBonus {
        unknown_int: u32,
        inverse_chance: u8,
        hp_10_perc: u8,
        hp_30_perc: u8,
        hp_50_perc: u8,
        ep_10_perc: u8,
        ep_30_perc: u8,
        ep_50_perc: u8,
        cp_10: u8,
        cp_30: u8,
        cp_50: u8,
        cp_100: u8,
        bp_1: u8,
        bp_3: u8,
        bp_5: u8,
        crit: u8,
        sepith_up: u8,
        zero_art: u8,
        zero_order: u8,
        guard_break: u8,
        status_attack: u8,
        deathblow: u8,
        vanish: u8,
        cp_200: u8,
        cp_0: u8,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "char_revise"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct CharRevise {
        pub char: String,
        pub hp: u16,
        pub str: u16,
        pub def: u16,
        pub ats: u16,
        pub adf: u16,
        pub spd: u16,
        pub brk: u16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "CharData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct CharData {
        pub unknown_shorts: [u16; 3],
        pub unknown_floats: [f32; 5],
        #[deserialize(length = 12)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "CompHelpData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct CompHelpData {
        pub r#type: u16,
        pub text: String,
        pub value: i32,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "game_difficulty"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct GameDifficulty {
        pub difficulty: u16,
        pub modifier: i16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "item"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct Item {
        pub id: u16,
        pub character_restriction: u16,
        pub flags: String,
        pub r#type: u16,
        pub subtype: u16,
        pub element: u16,
        pub slash: u8,
        pub thrust: u8,
        pub pierce: u8,
        pub strike: u8,
        pub target_type: u8,
        pub range: f32,
        pub area: u8,
        pub unknown_float_1: f32,
        pub unknown_float_2: f32,
        pub unknown_float_3: f32,
        pub effects: [Effect; 5],
        pub str: i16,
        pub def: i16,
        pub ats: i16,
        pub adf: i16,
        pub acc: i16,
        pub eva: i16,
        pub spd: i16,
        pub mov: i16,
        pub hp: i16,
        pub ep: i16,
        pub price: u32,
        pub carry_limit: u8,
        pub sort_id: u16,
        pub unknown_short: u16,
        pub name: String,
        pub description: String,
        #[deserialize(length = 8)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "magic"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct Magic {
        pub id: u16,
        pub character_restriction: u16,
        pub flags: String,
        pub category: u8,
        pub r#type: u8,
        pub element: u8,
        pub juna_specific: u8,
        pub target_type: u8,
        pub target_range: f32,
        pub target_size: u8,
        pub unknown_float_1: f32,
        pub unknown_float_2: f32,
        pub unknown_float_3: f32,
        pub effects: [Effect; 5],
        pub cast_delay: u8,
        pub recovery_delay: u8,
        pub cost: u16,
        pub unbalance_bonus: u8,
        pub break_bonus: u16,
        pub level_learn: u8,
        pub sort_id: u16,
        pub animation: String,
        pub name: String,
        pub description: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "magicbo"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MagicBo {
        pub id: u16,
        pub effects: [Effect; 5],
        pub turns: u16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MasterQuartzDummy"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MasterQuartzDummy {
        pub dummy_id: u16,
        pub original_id: u16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "se"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct SE {
        pub id: u16,
        pub unknown_string_1: String,
        pub unknown_string_2: String,
        #[deserialize(length = 2)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub unknown_float_1: f32,
        pub unknown_float_2: f32,
        #[deserialize(length = 4)]
        pub unknown_data_2: std::vec::Vec<u8>,
        pub unknown_float_3: f32,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "SlotCost"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct SlotCost {
        pub slot_position: u16,
        pub level: u16,
        pub earth: u16,
        pub water: u16,
        pub fire: u16,
        pub wind: u16,
        pub time: u16,
        pub space: u16,
        pub mirage: u16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "status"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct Status {
        pub script: String,
        pub texture: String,
        pub model: String,
        pub model_scale: f32,
        pub camera_pivot_height: f32,
        pub hitbox_size: f32,
        pub unknown_float_1: f32,
        pub unknown_float_2: f32,
        pub unknown_float_3: f32,
        pub unknown_float_4: f32,
        pub unknown_short_1: u16,
        pub unknown_short_2: u16,
        pub is_female: u8,
        pub level: u8,
        pub hp_base: u32,
        pub hp_growth: f32,
        pub max_ep: u16,
        pub start_ep: u16,
        pub max_cp: u16,
        pub start_cp: u16,
        pub str_base: u16,
        pub str_growth: f32,
        pub def_base: u16,
        pub def_growth: f32,
        pub ats_base: u16,
        pub ats_growth: f32,
        pub adf_base: u16,
        pub adf_growth: f32,
        pub dex_base: u16,
        pub dex_growth: f32,
        pub agl_base: u16,
        pub agl_growth: f32,
        pub eva: u16,
        pub spd_base: u16,
        pub spd_growth: f32,
        pub mov_base: u16,
        pub mov_growth: f32,
        pub exp_base: u16,
        pub exp_growth: f32,
        pub brk_mult_base: u16,
        pub brk_mult_growth: f32,
        pub earth_efficacy: u8,
        pub water_efficacy: u8,
        pub fire_efficacy: u8,
        pub wind_efficacy: u8,
        pub time_efficacy: u8,
        pub space_efficacy: u8,
        pub mirage_efficacy: u8,
        pub psn_efficacy: u8,
        pub seal_efficacy: u8,
        pub mute_efficacy: u8,
        pub blnd_efficacy: u8,
        pub slp_efficacy: u8,
        pub burn_efficacy: u8,
        pub frz_efficacy: u8,
        pub petr_efficacy: u8,
        pub fnt_efficacy: u8,
        pub conf_efficacy: u8,
        pub chrm_efficacy: u8,
        pub dblw_efficacy: u8,
        pub nmr_efficacy: u8,
        pub dlay_efficacy: u8,
        pub vnsh_efficacy: u8,
        pub s_dwn_efficacy: u8,
        pub slash_efficacy: u16,
        pub thrust_efficacy: u16,
        pub pierce_efficacy: u16,
        pub strike_efficacy: u16,
        pub earth_sepith_base: u8,
        pub water_sepith_base: u8,
        pub fire_sepith_base: u8,
        pub wind_sepith_base: u8,
        pub time_sepith_base: u8,
        pub space_sepith_base: u8,
        pub mirage_sepith_base: u8,
        pub sepith_mass_base: u8,
        pub earth_sepith_growth: f32,
        pub water_sepith_growth: f32,
        pub fire_sepith_growth: f32,
        pub wind_sepith_growth: f32,
        pub time_sepith_growth: f32,
        pub space_sepith_growth: f32,
        pub mirage_sepith_growth: f32,
        pub sepith_mass_growth: f32,
        pub drop_id_1: u16,
        pub drop_chance_1: u8,
        pub drop_id_2: u16,
        pub drop_chance_2: u8,
        pub stat_variation_min: f32,
        pub stat_variation_max: f32,
        pub flags: String,
        pub name: String,
        pub description: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "status_p"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct StatusP {
        pub id: u16,
        pub script: String,
        pub texture: String,
        pub model: String,
        pub model_scale: f32,
        pub camera_pivot_height: f32,
        pub hitbox_size: f32,
        pub unknown_float_1: f32,
        pub unknown_float_2: f32,
        pub unknown_float_3: f32,
        pub unknown_float_4: f32,
        pub unknown_short_1: u16,
        pub unknown_short_2: u16,
        pub is_female: u8,
        pub level: u8,
        pub hp_base: u32,
        pub hp_growth: f32,
        pub max_ep: u16,
        pub start_ep: u16,
        pub max_cp: u16,
        pub start_cp: u16,
        pub str_base: u16,
        pub str_growth: f32,
        pub def_base: u16,
        pub def_growth: f32,
        pub ats_base: u16,
        pub ats_growth: f32,
        pub adf_base: u16,
        pub adf_growth: f32,
        pub dex_base: u16,
        pub dex_growth: f32,
        pub agl_base: u16,
        pub agl_growth: f32,
        pub eva: u16,
        pub spd_base: u16,
        pub spd_growth: f32,
        pub mov_base: u16,
        pub mov_growth: f32,
        pub exp_base: u16,
        pub exp_growth: f32,
        pub brk_mult_base: u16,
        pub brk_mult_growth: f32,
        pub earth_efficacy: u8,
        pub water_efficacy: u8,
        pub fire_efficacy: u8,
        pub wind_efficacy: u8,
        pub time_efficacy: u8,
        pub space_efficacy: u8,
        pub mirage_efficacy: u8,
        pub psn_efficacy: u8,
        pub seal_efficacy: u8,
        pub mute_efficacy: u8,
        pub blnd_efficacy: u8,
        pub slp_efficacy: u8,
        pub burn_efficacy: u8,
        pub frz_efficacy: u8,
        pub petr_efficacy: u8,
        pub fnt_efficacy: u8,
        pub conf_efficacy: u8,
        pub chrm_efficacy: u8,
        pub dblw_efficacy: u8,
        pub nmr_efficacy: u8,
        pub dlay_efficacy: u8,
        pub vnsh_efficacy: u8,
        pub s_dwn_efficacy: u8,
        pub slash_efficacy: u16,
        pub thrust_efficacy: u16,
        pub pierce_efficacy: u16,
        pub strike_efficacy: u16,
        pub earth_sepith_base: u8,
        pub water_sepith_base: u8,
        pub fire_sepith_base: u8,
        pub wind_sepith_base: u8,
        pub time_sepith_base: u8,
        pub space_sepith_base: u8,
        pub mirage_sepith_base: u8,
        pub sepith_mass_base: u8,
        pub earth_sepith_growth: f32,
        pub water_sepith_growth: f32,
        pub fire_sepith_growth: f32,
        pub wind_sepith_growth: f32,
        pub time_sepith_growth: f32,
        pub space_sepith_growth: f32,
        pub mirage_sepith_growth: f32,
        pub sepith_mass_growth: f32,
        pub drop_id_1: u16,
        pub drop_chance_1: u8,
        pub drop_id_2: u16,
        pub drop_chance_2: u8,
        pub stat_variation_min: f32,
        pub stat_variation_max: f32,
        pub flags: String,
        pub name: String,
        pub description: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "voice"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct Voice {
        pub id: u16,
        pub unknown_byte: u8,
        pub clip: String,
        pub unknown_short_2: u16,
        pub unknown_float: f32,
        #[deserialize(length = 12)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = ""))]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, SerializeLength)]
    pub struct Effect {
        pub id: u16,
        pub data: [i32; 3],
    }
}

impl Entry {
    pub fn header_name(&self) -> &str {
        match self {
            Self::Generic(entry_data::Generic { header, .. }) => header,
            Self::ActiveVoiceTableData(_) => "ActiveVoiceTableData",
            Self::AtBonus(_) => "AtBonus",
            Self::CharRevise(_) => "char_revise",
            Self::CharData(_) => "CharData",
            Self::CompHelpData(_) => "CompHelpData",
            Self::GameDifficulty(_) => "game_difficulty",
            Self::Item(_) => "item",
            Self::Magic(_) => "magic",
            Self::MagicBo(_) => "magicbo",
            Self::MasterQuartzDummy(_) => "MasterQuartzDummy",
            Self::SE(_) => "se",
            Self::SlotCost(_) => "SlotCost",
            Self::Status(_) => "status",
            Self::StatusP(_) => "status_p",
            Self::Voice(_) => "voice",
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

impl ioutil::tbl::Tbl<Entry> for Tbl {
    fn new(headers: std::vec::Vec<String>, entries: std::vec::Vec<Entry>) -> Self {
        Self { headers, entries }
    }
}

#[derive(Debug)]
pub enum SerializeError {
    // TODO: number of offending entry
    TooMany { which: String, max: usize },
    EntryHeaderNotInHeaderList { which: String },
    EntryTooLong { header: String },
    UnexpectedEntryLength { header: String, length_prefix: u16, parsed_length: Option<u16> },
    WriteError(std::io::Error),
}

pub fn load(input: &[u8]) -> Result<Tbl, ParseError> {
    type ParseFun = ioutil::tbl::EntryParseFun<Entry>;
    lazy_static::lazy_static! {
        static ref PARSERS: std::collections::HashMap<&'static str, ParseFun> = [
                ("ActiveVoiceTableData", (|ctx| entry_data::ActiveVoiceTableData::deserialize(ctx).map(|(data, ctx)| (Entry::ActiveVoiceTableData(data), ctx))) as ParseFun),
                ("AtBonus", (|ctx| entry_data::AtBonus::deserialize(ctx).map(|(data, ctx)| (Entry::AtBonus(data), ctx))) as ParseFun),
                ("char_revise", (|ctx| entry_data::CharRevise::deserialize(ctx).map(|(data, ctx)| (Entry::CharRevise(data), ctx))) as ParseFun),
                ("CharData", (|ctx| entry_data::CharData::deserialize(ctx).map(|(data, ctx)| (Entry::CharData(data), ctx))) as ParseFun),
                ("CompHelpData", (|ctx| entry_data::CompHelpData::deserialize(ctx).map(|(data, ctx)| (Entry::CompHelpData(data), ctx))) as ParseFun),
                ("game_difficulty", (|ctx| entry_data::GameDifficulty::deserialize(ctx).map(|(data, ctx)| (Entry::GameDifficulty(data), ctx))) as ParseFun),
                ("item", (|ctx| entry_data::Item::deserialize(ctx).map(|(data, ctx)| (Entry::Item(data), ctx))) as ParseFun),
                ("magic", (|ctx| entry_data::Magic::deserialize(ctx).map(|(data, ctx)| (Entry::Magic(data), ctx))) as ParseFun),
                ("magicbo", (|ctx| entry_data::MagicBo::deserialize(ctx).map(|(data, ctx)| (Entry::MagicBo(data), ctx))) as ParseFun),
                ("MasterQuartzDummy", (|ctx| entry_data::MasterQuartzDummy::deserialize(ctx).map(|(data, ctx)| (Entry::MasterQuartzDummy(data), ctx))) as ParseFun),
                ("se", (|ctx| entry_data::SE::deserialize(ctx).map(|(data, ctx)| (Entry::SE(data), ctx))) as ParseFun),
                ("SlotCost", (|ctx| entry_data::SlotCost::deserialize(ctx).map(|(data, ctx)| (Entry::SlotCost(data), ctx))) as ParseFun),
                ("status", (|ctx| entry_data::Status::deserialize(ctx).map(|(data, ctx)| (Entry::Status(data), ctx))) as ParseFun),
                ("status_p", (|ctx| entry_data::StatusP::deserialize(ctx).map(|(data, ctx)| (Entry::StatusP(data), ctx))) as ParseFun),
                ("voice", (|ctx| entry_data::Voice::deserialize(ctx).map(|(data, ctx)| (Entry::Voice(data), ctx))) as ParseFun),
            ].into_iter().collect();
    }
    ioutil::tbl::parse_tbl::<Entry, Tbl>(&PARSERS, input)
}

pub fn store(tbl: &Tbl) -> Result<std::vec::Vec<u8>, SerializeError> {
    // header
    let mut data: std::vec::Vec<u8> = std::vec::Vec::new();
    let total_number_of_entries: u16 = tbl.entries.len().try_into().map_err(|_| SerializeError::TooMany {
        which: "total number of entries".to_string(),
        max: std::u16::MAX as usize,
    })?;
    let number_of_headers: u32 = tbl.headers.len().try_into().map_err(|_| SerializeError::TooMany {
        which: "number of headers".to_string(),
        max: std::u32::MAX as usize,
    })?;
    data.extend_from_slice(&total_number_of_entries.to_le_bytes());
    data.extend_from_slice(&number_of_headers.to_le_bytes());
    let mut header_counts: std::collections::HashMap<&str, u32> = tbl.headers.iter().map(|h| (h.as_str(), 0)).collect();
    for entry in tbl.entries.iter() {
        match header_counts.get_mut(entry.header_name()) {
            Some(n) => *n = n.checked_add(1).expect("should be smaller than total_number_of_entries which fits in a u16"),
            None => {
                return Err(SerializeError::EntryHeaderNotInHeaderList {
                    which: entry.header_name().to_string(),
                })
            }
        };
    }
    let header_counts = header_counts;
    for h in tbl.headers.iter() {
        let n = header_counts.get(h.as_str()).expect("we initialized every header count to 0");
        data.extend_from_slice(h.as_bytes());
        data.push(0);
        data.extend_from_slice(&n.to_le_bytes());
    }
    // actual data
    for e in tbl.entries.iter() {
        serialize_entry(&mut data, e)?;
    }
    Ok(data)
}

fn serialize_entry<W>(out: &mut W, entry: &Entry) -> Result<(), SerializeError>
where
    W: std::io::Write,
{
    // header name
    ioutil::serialize_utf8cstring(out, entry.header_name()).map_err(SerializeError::WriteError)?;
    // data length
    out.write(
        &entry
            .data_length()
            .ok_or_else(|| SerializeError::EntryTooLong {
                header: entry.header_name().to_string(),
            })?
            .to_le_bytes(),
    )
    .map_err(SerializeError::WriteError)?;
    // data
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
                (hex_literal::hex!("711db0f9f27f55cfbd99b0b2b556b94d5fe428efe2e4fb34a0f217fd5a08b40d68b6c959870a95a16336f5e05312742fb39dbb0e4ca921e0c3652aacbff81f41"), fix_CS3_PC_1_05_data_text_dat_en_t_name_tbl as FixFun),
                (hex_literal::hex!("6467d5835a0197ef06bf585cbc66d735aa808f6d1029c999c719cfc53ce3832dec92ce25845fb4f27c2ec5c0030020571e2e1a4b3977dba8fee4e413dd64f4fa"), fix_CS3_PC_1_05_data_text_dat_en_t_quest_tbl as FixFun),
                (hex_literal::hex!("7db08a20e3e887e7ad94f5ee1023e7c0130ebaca23f4e9715f5517192b6ab57dfbb60826e12e9e22e4293f9d28c6150189d14ea946b19a640e969958c88b87d3"), fix_CS3_PC_1_05_data_text_dat_fr_t_quest_tbl as FixFun),
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
fn fix_CS3_PC_1_05_data_text_dat_en_t_name_tbl(data: &mut std::vec::Vec<u8>) {
    // CS3 PC version 1.05, data/text/dat_en/t_name.tbl
    assert_eq!(data.len(), 150070);
    let replace = [(22392, 0x57, 0x55), (22493, 0x57, 0x55), (23898, 0x55, 0x53), (23997, 0x54, 0x52)];
    for (location, expected, replacement) in replace {
        replace_byte(data, location, expected, replacement);
    }
}

#[allow(non_snake_case)]
fn fix_CS3_PC_1_05_data_text_dat_en_t_quest_tbl(data: &mut std::vec::Vec<u8>) {
    // CS3 PC version 1.05, data/text/dat_en/t_quest.tbl
    assert_eq!(data.len(), 58017);
    let replace = [
        (914, 0x2c, 0x30),
        (972, 0x35, 0x39),
        (1039, 0x34, 0x38),
        (1105, 0x2f, 0x33),
        (1166, 0x2b, 0x2f),
        (1223, 0x33, 0x37),
        (1288, 0x2c, 0x30),
        (1346, 0x2c, 0x30),
        (1404, 0x32, 0x36),
        (1468, 0x2b, 0x2f),
        (1525, 0x2e, 0x32),
        (1585, 0x2d, 0x31),
        (1644, 0x34, 0x38),
        (1710, 0x36, 0x3a),
        (1778, 0x2e, 0x32),
        (1838, 0x34, 0x38),
        (1904, 0x34, 0x38),
        (1970, 0x35, 0x39),
        (2037, 0x33, 0x37),
        (2102, 0x26, 0x2a),
        (2154, 0x2d, 0x31),
        (2213, 0x2e, 0x32),
        (2273, 0x35, 0x39),
        (2340, 0x34, 0x38),
        (2406, 0x2d, 0x31),
        (2465, 0x36, 0x3a),
        (2533, 0x36, 0x3a),
        (2601, 0x30, 0x34),
        (2663, 0x36, 0x3a),
        (2731, 0x35, 0x39),
        (2798, 0x34, 0x38),
        (2864, 0x34, 0x38),
        (2930, 0x35, 0x39),
        (2997, 0x36, 0x3a),
        (3065, 0x28, 0x2c),
        (3119, 0x31, 0x35),
        (3182, 0x35, 0x39),
        (3249, 0x35, 0x39),
        (3316, 0x35, 0x39),
        (3383, 0x35, 0x39),
        (3450, 0x35, 0x39),
        (3517, 0x2f, 0x33),
    ];
    for (location, expected, replacement) in replace {
        replace_byte(data, location, expected, replacement);
    }
}

#[allow(non_snake_case)]
fn fix_CS3_PC_1_05_data_text_dat_fr_t_quest_tbl(data: &mut std::vec::Vec<u8>) {
    // CS3 PC version 1.05, data/text/dat_fr/t_quest.tbl
    assert_eq!(data.len(), 52904);
    let replace = [
        (921, 0x27, 0x2b),
        (974, 0x36, 0x3a),
        (1042, 0x39, 0x3d),
        (1113, 0x33, 0x37),
        (1178, 0x34, 0x38),
        (1244, 0x34, 0x38),
        (1310, 0x32, 0x36),
        (1374, 0x37, 0x3b),
        (1443, 0x31, 0x35),
        (1506, 0x34, 0x38),
        (1572, 0x37, 0x3b),
        (1641, 0x38, 0x3c),
        (1711, 0x35, 0x39),
        (1778, 0x33, 0x37),
        (1843, 0x36, 0x3a),
        (1911, 0x31, 0x35),
        (1974, 0x36, 0x3a),
        (2042, 0x32, 0x36),
        (2106, 0x37, 0x3b),
        (2175, 0x33, 0x37),
        (2240, 0x39, 0x3d),
        (2311, 0x34, 0x38),
        (2377, 0x38, 0x3c),
        (2447, 0x31, 0x35),
        (2510, 0x35, 0x39),
        (2577, 0x34, 0x38),
        (2643, 0x34, 0x38),
        (2709, 0x2d, 0x31),
        (2768, 0x35, 0x39),
        (2835, 0x34, 0x38),
        (2901, 0x38, 0x3c),
        (2971, 0x3a, 0x3e),
        (3043, 0x28, 0x2c),
        (3097, 0x2d, 0x31),
        (3156, 0x2c, 0x30),
        (3214, 0x38, 0x3c),
        (3284, 0x38, 0x3c),
        (3354, 0x38, 0x3c),
        (3424, 0x38, 0x3c),
        (3494, 0x38, 0x3c),
        (3564, 0x37, 0x3b),
        (3633, 0x33, 0x37),
    ];
    for (location, expected, replacement) in replace {
        replace_byte(data, location, expected, replacement);
    }
}

#[cfg(test)]
mod test {
    use super::{fix, load, store};
    extern crate walkdir;

    #[test]
    #[cfg(feature = "roundtrip-cs3")]
    fn test_roundtrip() {
        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs3/data/text/"));
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                println!("{}", path.display());
                let fixed = fix(&std::fs::read(path).unwrap());
                let tbl = load(&fixed).unwrap();
                let serialized = store(&tbl).unwrap();
                assert_eq!(serialized, fixed);
            }
        }
    }
}
