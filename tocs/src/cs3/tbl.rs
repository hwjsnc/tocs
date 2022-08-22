extern crate blake2;
extern crate hex_literal;
extern crate ioutil;
extern crate lazy_static;
extern crate tbl_deserialize_derive;
extern crate tbl_serialize_derive;
extern crate tbl_serialize_length_derive;

use ioutil::tbl::ParseError;

use ioutil::tbl::Deserialize as _;
use ioutil::tbl::Serialize as _;
use ioutil::tbl::SerializeLength as _;

// reexport for use outside of the crate
pub use ioutil::tbl::TblEntry;

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
    AssetCache(entry_data::AssetCache),
    AtBonus(entry_data::AtBonus),
    AttachTableData(entry_data::AttachTableData),
    Bgm(entry_data::Bgm),
    CharRevise(entry_data::CharRevise),
    CharData(entry_data::CharData),
    CompHelpData(entry_data::CompHelpData),
    EventTableData(entry_data::EventTableData),
    FootStepData(entry_data::FootStepData),
    GameDifficulty(entry_data::GameDifficulty),
    HkitugiLst(entry_data::HkitugiLst),
    Item(entry_data::Item),
    ItemQ(entry_data::ItemQ),
    ItemHelpData(entry_data::ItemHelpData),
    KisinPilotData(entry_data::KisinPilotData),
    LinkAbText(entry_data::LinkAbText),
    Magic(entry_data::Magic),
    MagicBo(entry_data::MagicBo),
    MapJumpData(entry_data::MapJumpData),
    MarkerTableData(entry_data::MarkerTableData),
    MasterQuartzBase(entry_data::MasterQuartzBase),
    MasterQuartzData(entry_data::MasterQuartzData),
    MasterQuartzDummy(entry_data::MasterQuartzDummy),
    MasterQuartzMemo(entry_data::MasterQuartzMemo),
    MasterQuartzStatus(entry_data::MasterQuartzStatus),
    MG08Card(entry_data::MG08Card),
    MG08CharMsg(entry_data::MG08CharMsg),
    MG08EnemyAI(entry_data::MG08EnemyAI),
    MG08PlyMsg(entry_data::MG08PlyMsg),
    MG08PresetDeck(entry_data::MG08PresetDeck),
    MG08Text(entry_data::MG08Text),
    NameTableData(entry_data::NameTableData),
    NameVis(entry_data::NameVis),
    NaviTextData(entry_data::NaviTextData),
    PlaceTableData(entry_data::PlaceTableData),
    PopupMsgFaceCharData(entry_data::PopupMsgFaceCharData),
    QSBook(entry_data::QSBook),
    QSChapter(entry_data::QSChapter),
    QSChar(entry_data::QSChar),
    QSCook(entry_data::QSCook),
    QSFish(entry_data::QSFish),
    QSHelp(entry_data::QSHelp),
    QSMons(entry_data::QSMons),
    QSRank(entry_data::QSRank),
    QSText(entry_data::QSText),
    QSTitle(entry_data::QSTitle),
    SchoolGrade(entry_data::SchoolGrade),
    SE(entry_data::SE),
    SlotCost(entry_data::SlotCost),
    Status(entry_data::Status),
    StatusP(entry_data::StatusP),
    TacticalBonus(entry_data::TacticalBonus),
    TextLst(entry_data::TextLst),
    TextTableData(entry_data::TextTableData),
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

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "AssetCache"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct AssetCache {
        #[deserialize(length = 2)]
        pub unknown_data: std::vec::Vec<u8>,
        pub unknown_string: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "AtBonus"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct AtBonus {
        pub unknown_int: u32,
        pub inverse_chance: u8,
        pub hp_10_perc: u8,
        pub hp_30_perc: u8,
        pub hp_50_perc: u8,
        pub ep_10_perc: u8,
        pub ep_30_perc: u8,
        pub ep_50_perc: u8,
        pub cp_10: u8,
        pub cp_30: u8,
        pub cp_50: u8,
        pub cp_100: u8,
        pub bp_1: u8,
        pub bp_3: u8,
        pub bp_5: u8,
        pub crit: u8,
        pub sepith_up: u8,
        pub zero_art: u8,
        pub zero_order: u8,
        pub guard_break: u8,
        pub status_attack: u8,
        pub deathblow: u8,
        pub vanish: u8,
        pub cp_200: u8,
        pub cp_0: u8,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "AttachTableData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct AttachTableData {
        pub character: u16,
        #[deserialize(length = 20)]
        pub unknown_data: std::vec::Vec<u8>,
        pub model: String,
        pub unknown_string: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "bgm"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct Bgm {
        pub id: u16,
        pub track: String,
        #[deserialize(length = 2)]
        pub unknown_data: std::vec::Vec<u8>,
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

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "EventTableData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct EventTableData {
        pub id: u16,
        pub unknown_string_1: String,
        pub unknown_string_2: String,
        #[deserialize(length = 6)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub unknown_string_3: String,
        #[deserialize(length = 18)]
        pub unknown_data_2: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "FootStepData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct FootStepData {
        pub unknown_string: String,
        pub unknown_byte: u8,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "game_difficulty"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct GameDifficulty {
        pub difficulty: u16,
        pub modifier: i16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "hkitugi_lst"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct HkitugiLst {
        pub unknown_short: u16,
        pub unknown_byte_1: u8,
        pub unknown_byte_2: u8,
        pub text: String,
        pub description: String,
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

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "item_q"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct ItemQ {
        pub id: u16,
        pub unknown_short_1: u16,
        pub flags: String,
        pub r#type: u16,
        pub subtype: u16,
        pub element: u16,
        #[deserialize(length = 4)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub quartz_type: u8,
        pub rating: f32,
        pub unknown_byte_1: u8,
        #[deserialize(length = 12)]
        pub unknown_data_2: std::vec::Vec<u8>,
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
        pub unknown_short_2: u16,
        pub name: String,
        pub description: String,
        pub prio_balanced: u16,
        pub prio_physical: u16,
        pub prio_magical: u16,
        pub prio_speed: u16,
        pub arts: [u16; 6],
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "ItemHelpData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct ItemHelpData {
        pub effect: u16,
        pub text: String,
        #[deserialize(length = 9)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "KisinPilotData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct KisinPilotData {
        #[deserialize(length = 8)]
        pub unknown_data: std::vec::Vec<u8>,
        pub unknown_string_1: String,
        pub unknown_string_2: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "LinkAbText"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct LinkAbText {
        pub id: u8,
        pub name: String,
        pub description: String,
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

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MapJumpData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MapJumpData {
        #[deserialize(length = 5)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub name: String,
        #[deserialize(length = 4)]
        pub unknown_data_2: std::vec::Vec<u8>,
        pub unknown_text_1: String,
        pub unknown_text_2: String,
        #[deserialize(length = 24)]
        pub unknown_data_3: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MarkerTableData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MarkerTableData {
        #[deserialize(length = 6)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub unknown_text_1: String,
        #[deserialize(length = 9)]
        pub unknown_data_2: std::vec::Vec<u8>,
        pub unknown_text_2: String,
        #[deserialize(length = 12)]
        pub unknown_data_3: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MasterQuartzBase"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MasterQuartzBase {
        pub item_id: u16,
        pub mq_id: u16,
        pub unknown_short: u16,
        pub hp_pattern: u16,
        pub ep_pattern: u16,
        pub str_pattern: u16,
        pub def_pattern: u16,
        pub ats_pattern: u16,
        pub adf_pattern: u16,
        pub spd_pattern: u16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MasterQuartzData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MasterQuartzData {
        pub item_id: u16,
        pub level: u16,
        pub effect_data: [f32; 9],
        pub arts: [i16; 2],
        pub memos: [u16; 9],
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MasterQuartzDummy"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MasterQuartzDummy {
        pub dummy_id: u16,
        pub original_id: u16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MasterQuartzMemo"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MasterQuartzMemo {
        pub item_id: u16,
        pub memo_id: u16,
        pub text: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MasterQuartzStatus"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MasterQuartzStatus {
        pub pattern_id: u8,
        pub level: u16,
        pub main_hp: i16,
        pub main_ep: i16,
        pub main_str: i16,
        pub main_def: i16,
        pub main_ats: i16,
        pub main_adf: i16,
        pub main_spd: i16,
        pub sub_hp: i16,
        pub sub_ep: i16,
        pub sub_str: i16,
        pub sub_def: i16,
        pub sub_ats: i16,
        pub sub_adf: i16,
        pub sub_spd: i16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MG08Card"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MG08Card {
        pub id: u16,
        pub r#type: u16,
        pub element: u16,
        pub name: String,

        pub item: u16,
        pub mana: u16,
        pub cost: u16,
        pub atk: u16,
        pub hp: u16,
        pub unknown_int: u16,

        pub ranged_flag: u16,
        pub ranged_string: String,

        pub flight_flag: u16,
        pub flight_string: String,

        pub skill_id: u16,
        pub skill_string: String,
        pub skill_cost: u16,
        pub skill_arg: u16,
        pub skill_description: String,

        pub ability_id: u16,
        pub ability_arg: u16,
        pub ability_description: String,

        pub effect_id: u16,
        pub unknown_int_2: u16,
        pub effect_arg_1: i16,
        pub effect_arg_2: i16,
        pub effect_description: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MG08CharMsg"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MG08CharMsg {
        pub character: u16,
        pub id: u16,
        pub unknown_int: u16,
        pub text: String,
        pub voice: i16,
        #[deserialize(length = 4)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MG08EnemyAi"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MG08EnemyAI {
        pub character: u16,
        pub id: u16,
        pub unknown_short_1: u16,
        pub unknown_short_2: u16,
        pub unknown_short_3: u16,
        pub unknown_short_4: u16,
        pub master: u16,
        pub cards: [u16; 20],
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MG08PlyMsg"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MG08PlyMsg {
        // this might be unused
        pub character: u16,
        pub id: u16,
        pub unknown_int: u16,
        pub text: String,
        pub voice: i16,
        #[deserialize(length = 4)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MG08PresetDeck"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MG08PresetDeck {
        pub id: u8,
        pub master: u8,
        pub cards: [u8; 20],
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "MG08Text"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct MG08Text {
        pub id: u16,
        pub text: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "NameTableData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct NameTableData {
        pub character: u16,
        pub name: String,
        pub model: String,
        pub animation: String,
        pub default_face: String,
        pub face: String,
        pub unknown_string: String,
        #[deserialize(length = 4)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub unknown_floats: [f32; 2],
        pub unknown_short_1: i16,
        pub unknown_short_2: i16,
        #[deserialize(length = 3)]
        pub unknown_data_2: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "NameVis"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct NameVis {
        pub unknown_string: String,
        #[deserialize(length = 8)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "NaviTextData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct NaviTextData {
        #[deserialize(length = 4)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub text: String,
        #[deserialize(length = 7)]
        pub unknown_data_2: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "PlaceTableData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct PlaceTableData {
        #[deserialize(length = 4)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub unknown_text_1: String,
        #[deserialize(length = 4)]
        pub unknown_data_2: std::vec::Vec<u8>,
        pub unknown_text_2: String,
        pub unknown_text_3: String,
        pub unknown_text_4: String,
        pub unknown_text_5: String,
        pub unknown_text_6: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "PopupMsgFaceCharData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct PopupMsgFaceCharData {
        pub unknown_short: u16,
        pub unknown_text_1: String,
        #[deserialize(length = 44)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSBook"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSBook {
        pub unknown_short_1: u16,
        pub unknown_short_2: u16,
        pub title: String,
        pub unknown_text_1: String,
        pub unknown_text_2: String,
        #[deserialize(length = 5)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSChapter"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSChapter {
        pub section_id: u16,
        pub section_name: String,
        pub unknown_short: u16,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSChar"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSChar {
        pub character: u16,
        #[deserialize(length = 6)]
        pub unknown_data: std::vec::Vec<u8>,
        pub unknown_text_1: String,
        pub name_short: String,
        pub name_long: String,
        pub affiliation_short: String,
        pub affiliation_long: String,
        pub note_1: String,
        pub note_2_title: String,
        pub note_2: String,
        pub note_3_title: String,
        pub note_3: String,
        pub note_4_title: String,
        pub note_4: String,
        pub note_5_title: String,
        pub note_5: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSCook"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSCook {
        pub regular_name: String,
        #[deserialize(length = 34)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub superb: QSCookVariantDescription,
        pub regular: QSCookVariantDescription,
        pub peculiar: QSCookVariantDescription,
        pub unique: QSCookVariantDescription,
        #[deserialize(length = 48)]
        pub unknown_data_4: std::vec::Vec<u8>, // TODO probably likelihoods to make each variant, from 0 (likely to be peculiar) to 4 (likely to be unique); needs testing tho
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSFish"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSFish {
        pub id: u16,
        pub unknown_string: String,
        pub name: String,
        #[deserialize(length = 167)]
        pub unknown_data_1: std::vec::Vec<u8>,
        pub description: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSHelp"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSHelp {
        pub unknown_short_1: u16,
        pub unknown_short_2: u16,
        pub text: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSMons"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSMons {
        pub enemy: String,
        pub unknown_string: String,
        pub section_id: u16,
        pub unknown_short: u16,
        pub is_extra: u8,
        pub unknown_byte: u8,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSRank"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSRank {
        pub id: u16,
        pub text: String,
        #[deserialize(length = 6)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSText"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSText {
        pub id: u16,
        pub unknown_byte_1: u8,
        pub text: String,
        pub unknown_byte_2: u8,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "QSTitle"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct QSTitle {
        pub unknown_short: u16,
        pub unknown_byte: u8,
        pub title: String,
        pub persons: String,
        #[deserialize(length = 13)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "SchoolGrade"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct SchoolGrade {
        pub id: u16,
        pub text: String,
        #[deserialize(length = 6)]
        pub unknown_data: std::vec::Vec<u8>,
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

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "TacticalBonus"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct TacticalBonus {
        pub id: u16,
        pub name: String,
        pub bonus: f32,
        #[deserialize(length = 2)]
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "text_lst"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct TextLst {
        pub id: u16,
        pub text: String,
    }

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = "TextTableData"))]
    #[derive(Clone, Debug, Deserialize, Serialize, SerializeLength)]
    pub struct TextTableData {
        pub id: u16,
        pub text: String,
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

    #[cfg_attr(feature = "sqliteio", derive(SqliteIO), sqlite(table = ""))]
    #[derive(Clone, Debug, Default, Deserialize, Serialize, SerializeLength)]
    pub struct QSCookVariantDescription {
        pub item_id: u16,
        pub description_1: String,
        pub description_2: String,
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
            Self::ActiveVoiceTableData(_) => "ActiveVoiceTableData",
            Self::AssetCache(_) => "AssetCache",
            Self::AtBonus(_) => "AtBonus",
            Self::AttachTableData(_) => "AttachTableData",
            Self::Bgm(_) => "bgm",
            Self::CharRevise(_) => "char_revise",
            Self::CharData(_) => "CharData",
            Self::CompHelpData(_) => "CompHelpData",
            Self::EventTableData(_) => "EventTableData",
            Self::FootStepData(_) => "FootStepData",
            Self::GameDifficulty(_) => "game_difficulty",
            Self::HkitugiLst(_) => "hkitugi_lst",
            Self::Item(_) => "item",
            Self::ItemQ(_) => "item_q",
            Self::ItemHelpData(_) => "ItemHelpData",
            Self::KisinPilotData(_) => "KisinPilotData",
            Self::LinkAbText(_) => "LinkAbText",
            Self::Magic(_) => "magic",
            Self::MagicBo(_) => "magicbo",
            Self::MapJumpData(_) => "MapJumpData",
            Self::MarkerTableData(_) => "MarkerTableData",
            Self::MasterQuartzBase(_) => "MasterQuartzBase",
            Self::MasterQuartzData(_) => "MasterQuartzData",
            Self::MasterQuartzDummy(_) => "MasterQuartzDummy",
            Self::MasterQuartzMemo(_) => "MasterQuartzMemo",
            Self::MasterQuartzStatus(_) => "MasterQuartzStatus",
            Self::MG08Card(_) => "MG08Card",
            Self::MG08CharMsg(_) => "MG08CharMsg",
            Self::MG08EnemyAI(_) => "MG08EnemyAi",
            Self::MG08PlyMsg(_) => "MG08PlyMsg",
            Self::MG08PresetDeck(_) => "MG08PresetDeck",
            Self::MG08Text(_) => "MG08Text",
            Self::NameTableData(_) => "NameTableData",
            Self::NameVis(_) => "NameVis",
            Self::NaviTextData(_) => "NaviTextData",
            Self::PlaceTableData(_) => "PlaceTableData",
            Self::PopupMsgFaceCharData(_) => "PopupMsgFaceCharData",
            Self::QSBook(_) => "QSBook",
            Self::QSChapter(_) => "QSChapter",
            Self::QSChar(_) => "QSChar",
            Self::QSCook(_) => "QSCook",
            Self::QSFish(_) => "QSFish",
            Self::QSHelp(_) => "QSHelp",
            Self::QSMons(_) => "QSMons",
            Self::QSRank(_) => "QSRank",
            Self::QSText(_) => "QSText",
            Self::QSTitle(_) => "QSTitle",
            Self::SchoolGrade(_) => "SchoolGrade",
            Self::SE(_) => "se",
            Self::SlotCost(_) => "SlotCost",
            Self::Status(_) => "status",
            Self::StatusP(_) => "status_p",
            Self::TacticalBonus(_) => "TacticalBonus",
            Self::TextTableData(_) => "TextTableData",
            Self::TextLst(_) => "text_lst",
            Self::Voice(_) => "voice",
        }
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
                ("AssetCache", (|ctx| entry_data::AssetCache::deserialize(ctx).map(|(data, ctx)| (Entry::AssetCache(data), ctx))) as ParseFun),
                ("AtBonus", (|ctx| entry_data::AtBonus::deserialize(ctx).map(|(data, ctx)| (Entry::AtBonus(data), ctx))) as ParseFun),
                ("AttachTableData", (|ctx| entry_data::AttachTableData::deserialize(ctx).map(|(data, ctx)| (Entry::AttachTableData(data), ctx))) as ParseFun),
                ("bgm", (|ctx| entry_data::Bgm::deserialize(ctx).map(|(data, ctx)| (Entry::Bgm(data), ctx))) as ParseFun),
                ("char_revise", (|ctx| entry_data::CharRevise::deserialize(ctx).map(|(data, ctx)| (Entry::CharRevise(data), ctx))) as ParseFun),
                ("CharData", (|ctx| entry_data::CharData::deserialize(ctx).map(|(data, ctx)| (Entry::CharData(data), ctx))) as ParseFun),
                ("CompHelpData", (|ctx| entry_data::CompHelpData::deserialize(ctx).map(|(data, ctx)| (Entry::CompHelpData(data), ctx))) as ParseFun),
                ("EventTableData", (|ctx| entry_data::EventTableData::deserialize(ctx).map(|(data, ctx)| (Entry::EventTableData(data), ctx))) as ParseFun),
                ("FootStepData", (|ctx| entry_data::FootStepData::deserialize(ctx).map(|(data, ctx)| (Entry::FootStepData(data), ctx))) as ParseFun),
                ("game_difficulty", (|ctx| entry_data::GameDifficulty::deserialize(ctx).map(|(data, ctx)| (Entry::GameDifficulty(data), ctx))) as ParseFun),
                ("hkitugi_lst", (|ctx| entry_data::HkitugiLst::deserialize(ctx).map(|(data, ctx)| (Entry::HkitugiLst(data), ctx))) as ParseFun),
                ("item", (|ctx| entry_data::Item::deserialize(ctx).map(|(data, ctx)| (Entry::Item(data), ctx))) as ParseFun),
                ("item_q", (|ctx| entry_data::ItemQ::deserialize(ctx).map(|(data, ctx)| (Entry::ItemQ(data), ctx))) as ParseFun),
                ("ItemHelpData", (|ctx| entry_data::ItemHelpData::deserialize(ctx).map(|(data, ctx)| (Entry::ItemHelpData(data), ctx))) as ParseFun),
                ("KisinPilotData", (|ctx| entry_data::KisinPilotData::deserialize(ctx).map(|(data, ctx)| (Entry::KisinPilotData(data), ctx))) as ParseFun),
                ("LinkAbText", (|ctx| entry_data::LinkAbText::deserialize(ctx).map(|(data, ctx)| (Entry::LinkAbText(data), ctx))) as ParseFun),
                ("magic", (|ctx| entry_data::Magic::deserialize(ctx).map(|(data, ctx)| (Entry::Magic(data), ctx))) as ParseFun),
                ("magicbo", (|ctx| entry_data::MagicBo::deserialize(ctx).map(|(data, ctx)| (Entry::MagicBo(data), ctx))) as ParseFun),
                ("MapJumpData", (|ctx| entry_data::MapJumpData::deserialize(ctx).map(|(data, ctx)| (Entry::MapJumpData(data), ctx))) as ParseFun),
                ("MarkerTableData", (|ctx| entry_data::MarkerTableData::deserialize(ctx).map(|(data, ctx)| (Entry::MarkerTableData(data), ctx))) as ParseFun),
                ("MasterQuartzBase", (|ctx| entry_data::MasterQuartzBase::deserialize(ctx).map(|(data, ctx)| (Entry::MasterQuartzBase(data), ctx))) as ParseFun),
                ("MasterQuartzData", (|ctx| entry_data::MasterQuartzData::deserialize(ctx).map(|(data, ctx)| (Entry::MasterQuartzData(data), ctx))) as ParseFun),
                ("MasterQuartzDummy", (|ctx| entry_data::MasterQuartzDummy::deserialize(ctx).map(|(data, ctx)| (Entry::MasterQuartzDummy(data), ctx))) as ParseFun),
                ("MasterQuartzMemo", (|ctx| entry_data::MasterQuartzMemo::deserialize(ctx).map(|(data, ctx)| (Entry::MasterQuartzMemo(data), ctx))) as ParseFun),
                ("MasterQuartzStatus", (|ctx| entry_data::MasterQuartzStatus::deserialize(ctx).map(|(data, ctx)| (Entry::MasterQuartzStatus(data), ctx))) as ParseFun),
                ("MG08Card", (|ctx| entry_data::MG08Card::deserialize(ctx).map(|(data, ctx)| (Entry::MG08Card(data), ctx))) as ParseFun),
                ("MG08CharMsg", (|ctx| entry_data::MG08CharMsg::deserialize(ctx).map(|(data, ctx)| (Entry::MG08CharMsg(data), ctx))) as ParseFun),
                ("MG08EnemyAi", (|ctx| entry_data::MG08EnemyAI::deserialize(ctx).map(|(data, ctx)| (Entry::MG08EnemyAI(data), ctx))) as ParseFun),
                ("MG08PlyMsg", (|ctx| entry_data::MG08PlyMsg::deserialize(ctx).map(|(data, ctx)| (Entry::MG08PlyMsg(data), ctx))) as ParseFun),
                ("MG08PresetDeck", (|ctx| entry_data::MG08PresetDeck::deserialize(ctx).map(|(data, ctx)| (Entry::MG08PresetDeck(data), ctx))) as ParseFun),
                ("MG08Text", (|ctx| entry_data::MG08Text::deserialize(ctx).map(|(data, ctx)| (Entry::MG08Text(data), ctx))) as ParseFun),
                ("NameTableData", (|ctx| entry_data::NameTableData::deserialize(ctx).map(|(data, ctx)| (Entry::NameTableData(data), ctx))) as ParseFun),
                ("NameVis", (|ctx| entry_data::NameVis::deserialize(ctx).map(|(data, ctx)| (Entry::NameVis(data), ctx))) as ParseFun),
                ("NaviTextData", (|ctx| entry_data::NaviTextData::deserialize(ctx).map(|(data, ctx)| (Entry::NaviTextData(data), ctx))) as ParseFun),
                ("PlaceTableData", (|ctx| entry_data::PlaceTableData::deserialize(ctx).map(|(data, ctx)| (Entry::PlaceTableData(data), ctx))) as ParseFun),
                ("PopupMsgFaceCharData", (|ctx| entry_data::PopupMsgFaceCharData::deserialize(ctx).map(|(data, ctx)| (Entry::PopupMsgFaceCharData(data), ctx))) as ParseFun),
                ("QSBook", (|ctx| entry_data::QSBook::deserialize(ctx).map(|(data, ctx)| (Entry::QSBook(data), ctx))) as ParseFun),
                ("QSChapter", (|ctx| entry_data::QSChapter::deserialize(ctx).map(|(data, ctx)| (Entry::QSChapter(data), ctx))) as ParseFun),
                ("QSChar", (|ctx| entry_data::QSChar::deserialize(ctx).map(|(data, ctx)| (Entry::QSChar(data), ctx))) as ParseFun),
                ("QSCook", (|ctx| entry_data::QSCook::deserialize(ctx).map(|(data, ctx)| (Entry::QSCook(data), ctx))) as ParseFun),
                ("QSFish", (|ctx| entry_data::QSFish::deserialize(ctx).map(|(data, ctx)| (Entry::QSFish(data), ctx))) as ParseFun),
                ("QSHelp", (|ctx| entry_data::QSHelp::deserialize(ctx).map(|(data, ctx)| (Entry::QSHelp(data), ctx))) as ParseFun),
                ("QSMons", (|ctx| entry_data::QSMons::deserialize(ctx).map(|(data, ctx)| (Entry::QSMons(data), ctx))) as ParseFun),
                ("QSRank", (|ctx| entry_data::QSRank::deserialize(ctx).map(|(data, ctx)| (Entry::QSRank(data), ctx))) as ParseFun),
                ("QSText", (|ctx| entry_data::QSText::deserialize(ctx).map(|(data, ctx)| (Entry::QSText(data), ctx))) as ParseFun),
                ("QSTitle", (|ctx| entry_data::QSTitle::deserialize(ctx).map(|(data, ctx)| (Entry::QSTitle(data), ctx))) as ParseFun),
                ("SchoolGrade", (|ctx| entry_data::SchoolGrade::deserialize(ctx).map(|(data, ctx)| (Entry::SchoolGrade(data), ctx))) as ParseFun),
                ("se", (|ctx| entry_data::SE::deserialize(ctx).map(|(data, ctx)| (Entry::SE(data), ctx))) as ParseFun),
                ("SlotCost", (|ctx| entry_data::SlotCost::deserialize(ctx).map(|(data, ctx)| (Entry::SlotCost(data), ctx))) as ParseFun),
                ("status", (|ctx| entry_data::Status::deserialize(ctx).map(|(data, ctx)| (Entry::Status(data), ctx))) as ParseFun),
                ("status_p", (|ctx| entry_data::StatusP::deserialize(ctx).map(|(data, ctx)| (Entry::StatusP(data), ctx))) as ParseFun),
                ("TacticalBonus", (|ctx| entry_data::TacticalBonus::deserialize(ctx).map(|(data, ctx)| (Entry::TacticalBonus(data), ctx))) as ParseFun),
                ("text_lst", (|ctx| entry_data::TextLst::deserialize(ctx).map(|(data, ctx)| (Entry::TextLst(data), ctx))) as ParseFun),
                ("TextTableData", (|ctx| entry_data::TextTableData::deserialize(ctx).map(|(data, ctx)| (Entry::TextTableData(data), ctx))) as ParseFun),
                ("voice", (|ctx| entry_data::Voice::deserialize(ctx).map(|(data, ctx)| (Entry::Voice(data), ctx))) as ParseFun),
            ].into_iter().collect();
    }
    let parse_options = ioutil::tbl::ParseOptions {
        with_header_counts: true,
        strict: true,
    };
    ioutil::tbl::parse_tbl::<Entry, Tbl>(&PARSERS, parse_options, input)
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
