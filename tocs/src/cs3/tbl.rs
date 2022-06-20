extern crate blake2;
extern crate hex_literal;
extern crate ioutil;
extern crate lazy_static;
extern crate tbl_serialize_derive;

use ioutil::serialize_utf8cstring;
use ioutil::tbl::{ParseError, ParseErrorContext, ParseErrorDetail};
use ioutil::{parse_bytes, parse_eof, parse_le_f32, parse_le_i16, parse_le_i32, parse_le_u16, parse_le_u32, parse_u8, parse_utf8cstring, ParseContext};

use ioutil::tbl::Deserialize as _;
use ioutil::tbl::Serialize as _;

use blake2::Digest as _;

use std::vec::Vec;

#[derive(Clone, Debug)]
pub struct Tbl {
    pub headers: Vec<String>,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
pub enum Entry {
    Generic(entry_data::Generic),
    CharRevise(entry_data::CharRevise),
    CompHelpData(entry_data::CompHelpData),
    Item(entry_data::Item),
    Magic(entry_data::Magic),
    MagicBo(entry_data::MagicBo),
    SlotCost(entry_data::SlotCost),
    Status(entry_data::Status),
    Voice(entry_data::Voice),
}

pub mod entry_data {
    use tbl_serialize_derive::Serialize;

    #[derive(Clone, Debug)]
    pub struct Generic {
        pub header: String,
        pub data: Vec<u8>,
    }

    #[derive(Clone, Debug)]
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

    #[derive(Clone, Debug)]
    pub struct CompHelpData {
        pub r#type: u16,
        pub text: String,
        pub value: i32,
    }

    #[derive(Clone, Debug)]
    pub struct Item {
        pub id: u16,
        pub character_restriction: u16,
        pub flags: String,
        pub r#type: u16,
        pub subtype: u16,
        pub element: u8,
        pub slash: u8,
        pub thrust: u8,
        pub pierce: u8,
        pub strike: u8,
        pub unknown_byte: u8,
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
        pub unknown_data: [u8; 8],
    }

    #[derive(Clone, Debug)]
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

    #[derive(Clone, Debug)]
    pub struct MagicBo {
        pub id: u16,
        pub effects: [Effect; 5],
        pub turns: u16,
    }

    #[derive(Clone, Debug, Serialize)]
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

    #[derive(Clone, Debug)]
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
        pub unknown_float_5: f32,
        pub unknown_float_6: f32,
        pub flags: String,
        pub name: String,
        pub description: String,
    }

    #[derive(Clone, Debug)]
    pub struct Voice {
        pub unknown_short_1: u16,
        pub unknown_byte: u8,
        pub clip: String,
        pub unknown_short_2: u16,
        pub unknown_float: f32,
        pub unknown_data: std::vec::Vec<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct Effect {
        pub id: u16,
        pub data: [i32; 3],
    }
}

impl Entry {
    pub fn header_name(&self) -> &str {
        match self {
            Self::Generic(entry_data::Generic { header, .. }) => header,
            Self::CharRevise(_) => "char_revise",
            Self::CompHelpData(_) => "CompHelpData",
            Self::Item(_) => "item",
            Self::Magic(_) => "magic",
            Self::MagicBo(_) => "magicbo",
            Self::SlotCost(_) => "SlotCost",
            Self::Status(_) => "status",
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
                ("char_revise", parse_char_revise_data as ParseFun),
                ("CompHelpData", parse_CompHelpData_data as ParseFun),
                ("item", parse_item_data as ParseFun),
                ("magic", parse_magic_data as ParseFun),
                ("magicbo", parse_magicbo_data as ParseFun),
                ("SlotCost", parse_SlotCost_data as ParseFun),
                ("status", parse_status_data as ParseFun),
                ("voice", parse_voice_data as ParseFun),
            ].into_iter().collect();
    }

    let (number_of_entries, header_names_with_counts, ctx) = ioutil::tbl::parse_tbl_header_v2(ParseContext::start_of(input))?;
    let (entries, ctx) = ioutil::tbl::parse_entries(&PARSERS, ctx, number_of_entries)?;
    // TODO: check that number of entries matches actual table entries
    // TODO: check that every entry is present in the headers
    if ctx.strict {
        let _ = parse_eof(ctx).map_err(|e| ParseError {
            detail: ParseErrorDetail::Primitive(e),
            context: ParseErrorContext::End,
        })?;
    }
    let tbl = Tbl {
        headers: header_names_with_counts.into_iter().map(|(h, _)| h.into_owned()).collect(),
        entries,
    };
    Ok(tbl)
}

fn parse_char_revise_data(ctx: ParseContext) -> Result<(Entry, ParseContext), ParseErrorDetail> {
    let (char, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (hp, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (str, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (def, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (ats, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (adf, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (spd, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (brk, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    Ok((
        Entry::CharRevise(entry_data::CharRevise {
            char: char.to_string(),
            hp,
            str,
            def,
            ats,
            adf,
            spd,
            brk,
        }),
        ctx,
    ))
}

#[allow(non_snake_case)]
fn parse_CompHelpData_data(ctx: ParseContext) -> Result<(Entry, ParseContext), ParseErrorDetail> {
    let (r#type, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (text, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (value, ctx) = parse_le_i32(ctx).map_err(ParseErrorDetail::Primitive)?;
    Ok((
        Entry::CompHelpData(entry_data::CompHelpData {
            r#type,
            text: text.to_string(),
            value,
        }),
        ctx,
    ))
}

fn parse_item_data(ctx: ParseContext) -> Result<(Entry, ParseContext), ParseErrorDetail> {
    let (id, ctx) = u16::deserialize(ctx)?;
    let (character_restriction, ctx) = u16::deserialize(ctx)?;
    let (flags, ctx) = String::deserialize(ctx)?;
    let (r#type, ctx) = u16::deserialize(ctx)?;
    let (subtype, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (element, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (slash, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (thrust, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (pierce, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (strike, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_byte, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (target_type, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (range, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (area, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_1, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_2, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_3, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (effects, ctx) = parse_effects(ctx)?;
    let (str, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (def, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (ats, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (adf, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (acc, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (eva, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (spd, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (mov, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (hp, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (ep, ctx) = parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (price, ctx) = parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (carry_limit, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (sort_id, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_short, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (name, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (description, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let mut ctx = ctx;
    let mut unknown_data: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0]; // TODO: replace with vector
    for x in unknown_data.iter_mut() {
        (*x, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
        /*
                TODO: unknown content error
                if ctx.strict && *x != 0 {
                if *x != 0
        n           return nom::combinator::fail(rest); // TODO: proper error code
                }}
                */
    }
    Ok((
        Entry::Item(entry_data::Item {
            id,
            character_restriction,
            flags: flags.to_string(),
            r#type,
            subtype,
            element,
            slash,
            thrust,
            pierce,
            strike,
            unknown_byte,
            target_type,
            range,
            area,
            unknown_float_1,
            unknown_float_2,
            unknown_float_3,
            effects,
            str,
            def,
            ats,
            adf,
            acc,
            eva,
            spd,
            mov,
            hp,
            ep,
            price,
            carry_limit,
            sort_id,
            unknown_short,
            name: name.to_string(),
            description: description.to_string(),
            unknown_data,
        }),
        ctx,
    ))
}

fn parse_magic_data(ctx: ParseContext) -> Result<(Entry, ParseContext), ParseErrorDetail> {
    let (id, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (character_restriction, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (flags, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (category, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (r#type, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (element, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (juna_specific, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (target_type, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (target_range, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (target_size, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_1, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_2, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_3, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (effects, ctx) = parse_effects(ctx)?;
    let (cast_delay, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (recovery_delay, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (cost, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unbalance_bonus, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (break_bonus, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (level_learn, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (sort_id, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (animation, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (name, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (description, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    Ok((
        Entry::Magic(entry_data::Magic {
            id,
            character_restriction,
            flags: flags.to_string(),
            category,
            r#type,
            element,
            juna_specific,
            target_type,
            target_range,
            target_size,
            unknown_float_1,
            unknown_float_2,
            unknown_float_3,
            effects,
            cast_delay,
            recovery_delay,
            cost,
            unbalance_bonus,
            break_bonus,
            level_learn,
            sort_id,
            animation: animation.to_string(),
            name: name.to_string(),
            description: description.to_string(),
        }),
        ctx,
    ))
}

fn parse_magicbo_data(ctx: ParseContext) -> Result<(Entry, ParseContext), ParseErrorDetail> {
    let (id, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (effects, ctx) = parse_effects(ctx)?;
    let (turns, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    Ok((Entry::MagicBo(entry_data::MagicBo { id, effects, turns }), ctx))
}

#[allow(non_snake_case)]
fn parse_SlotCost_data(ctx: ParseContext) -> Result<(Entry, ParseContext), ParseErrorDetail> {
    let (slot_position, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (level, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (earth, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (water, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (fire, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (wind, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (time, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (space, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (mirage, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    Ok((
        Entry::SlotCost(entry_data::SlotCost {
            slot_position,
            level,
            earth,
            water,
            fire,
            wind,
            time,
            space,
            mirage,
        }),
        ctx,
    ))
}

fn parse_status_data(ctx: ParseContext) -> Result<(Entry, ParseContext), ParseErrorDetail> {
    let (script, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (texture, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (model, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (model_scale, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (camera_pivot_height, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (hitbox_size, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_1, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_2, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_3, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_4, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_short_1, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_short_2, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (is_female, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (level, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (hp_base, ctx) = parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (hp_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (max_ep, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (start_ep, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (max_cp, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (start_cp, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (str_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (str_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (def_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (def_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (ats_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (ats_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (adf_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (adf_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (dex_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (dex_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (agl_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (agl_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (eva, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (spd_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (spd_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (mov_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (mov_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (exp_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (exp_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (brk_mult_base, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (brk_mult_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (earth_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (water_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (fire_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (wind_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (time_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (space_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (mirage_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (psn_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (seal_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (mute_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (blnd_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (slp_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (burn_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (frz_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (petr_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (fnt_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (conf_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (chrm_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (dblw_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (nmr_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (dlay_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (vnsh_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (s_dwn_efficacy, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (slash_efficacy, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (thrust_efficacy, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (pierce_efficacy, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (strike_efficacy, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (earth_sepith_base, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (water_sepith_base, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (fire_sepith_base, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (wind_sepith_base, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (time_sepith_base, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (space_sepith_base, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (mirage_sepith_base, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (sepith_mass_base, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (earth_sepith_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (water_sepith_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (fire_sepith_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (wind_sepith_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (time_sepith_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (space_sepith_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (mirage_sepith_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (sepith_mass_growth, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (drop_id_1, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (drop_chance_1, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (drop_id_2, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (drop_chance_2, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_5, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float_6, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (flags, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (name, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (description, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;

    Ok((
        Entry::Status(entry_data::Status {
            script: script.into_owned(),
            texture: texture.into_owned(),
            model: model.into_owned(),
            model_scale,
            camera_pivot_height,
            hitbox_size,
            unknown_float_1,
            unknown_float_2,
            unknown_float_3,
            unknown_float_4,
            unknown_short_1,
            unknown_short_2,
            is_female,
            level,
            hp_base,
            hp_growth,
            max_ep,
            start_ep,
            max_cp,
            start_cp,
            str_base,
            str_growth,
            def_base,
            def_growth,
            ats_base,
            ats_growth,
            adf_base,
            adf_growth,
            dex_base,
            dex_growth,
            agl_base,
            agl_growth,
            eva,
            spd_base,
            spd_growth,
            mov_base,
            mov_growth,
            exp_base,
            exp_growth,
            brk_mult_base,
            brk_mult_growth,
            earth_efficacy,
            water_efficacy,
            fire_efficacy,
            wind_efficacy,
            time_efficacy,
            space_efficacy,
            mirage_efficacy,
            psn_efficacy,
            seal_efficacy,
            mute_efficacy,
            blnd_efficacy,
            slp_efficacy,
            burn_efficacy,
            frz_efficacy,
            petr_efficacy,
            fnt_efficacy,
            conf_efficacy,
            chrm_efficacy,
            dblw_efficacy,
            nmr_efficacy,
            dlay_efficacy,
            vnsh_efficacy,
            s_dwn_efficacy,
            slash_efficacy,
            thrust_efficacy,
            pierce_efficacy,
            strike_efficacy,
            earth_sepith_base,
            water_sepith_base,
            fire_sepith_base,
            wind_sepith_base,
            time_sepith_base,
            space_sepith_base,
            mirage_sepith_base,
            sepith_mass_base,
            earth_sepith_growth,
            water_sepith_growth,
            fire_sepith_growth,
            wind_sepith_growth,
            time_sepith_growth,
            space_sepith_growth,
            mirage_sepith_growth,
            sepith_mass_growth,
            drop_id_1,
            drop_chance_1,
            drop_id_2,
            drop_chance_2,
            unknown_float_5,
            unknown_float_6,
            flags: flags.into_owned(),
            name: name.into_owned(),
            description: description.into_owned(),
        }),
        ctx,
    ))
}

fn parse_voice_data(ctx: ParseContext) -> Result<(Entry, ParseContext), ParseErrorDetail> {
    let (unknown_short_1, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_byte, ctx) = parse_u8(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (clip, ctx) = parse_utf8cstring(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_short_2, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_float, ctx) = parse_le_f32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (unknown_data, ctx) = parse_bytes(ctx, 12).map_err(ParseErrorDetail::Primitive)?;
    Ok((
        Entry::Voice(entry_data::Voice {
            unknown_short_1,
            unknown_byte,
            clip: clip.to_string(),
            unknown_short_2,
            unknown_float,
            unknown_data: unknown_data.to_vec(),
        }),
        ctx,
    ))
}

fn parse_effects(ctx: ParseContext) -> Result<([entry_data::Effect; 5], ParseContext), ParseErrorDetail> {
    let mut ctx = ctx;
    let mut effects = [
        entry_data::Effect { id: 0, data: [0, 0, 0] },
        entry_data::Effect { id: 0, data: [0, 0, 0] },
        entry_data::Effect { id: 0, data: [0, 0, 0] },
        entry_data::Effect { id: 0, data: [0, 0, 0] },
        entry_data::Effect { id: 0, data: [0, 0, 0] },
    ];
    for eff in &mut effects {
        (eff.id, ctx) = parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)?;
        for x in &mut eff.data {
            (*x, ctx) = parse_le_i32(ctx).map_err(ParseErrorDetail::Primitive)?;
        }
    }
    Ok((effects, ctx))
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
    // header name & size
    ioutil::serialize_utf8cstring(out, entry.header_name()).map_err(SerializeError::WriteError)?;
    out.write(
        &serialized_data_length(entry)
            .ok_or_else(|| SerializeError::EntryTooLong {
                header: entry.header_name().to_string(),
            })?
            .to_le_bytes(),
    )
    .map_err(SerializeError::WriteError)?;
    // data
    match entry {
        Entry::Generic(entry_data::Generic { data, .. }) => {
            out.write(data).map_err(SerializeError::WriteError)?;
        }
        Entry::CharRevise(entry_data::CharRevise {
            char,
            hp,
            str,
            def,
            ats,
            adf,
            spd,
            brk,
        }) => {
            ioutil::serialize_utf8cstring(out, char).map_err(SerializeError::WriteError)?;
            out.write(&hp.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&str.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&def.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&ats.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&adf.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&spd.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&brk.to_le_bytes()).map_err(SerializeError::WriteError)?;
        }
        Entry::CompHelpData(entry_data::CompHelpData { r#type, text, value }) => {
            out.write(&r#type.to_le_bytes()).map_err(SerializeError::WriteError)?;
            ioutil::serialize_utf8cstring(out, text).map_err(SerializeError::WriteError)?;
            out.write(&value.to_le_bytes()).map_err(SerializeError::WriteError)?;
        }
        Entry::Item(entry_data::Item {
            id,
            character_restriction,
            flags,
            r#type,
            subtype,
            element,
            slash,
            thrust,
            pierce,
            strike,
            unknown_byte,
            target_type,
            range,
            area,
            unknown_float_1,
            unknown_float_2,
            unknown_float_3,
            effects,
            str,
            def,
            ats,
            adf,
            acc,
            eva,
            spd,
            mov,
            hp,
            ep,
            price,
            carry_limit,
            sort_id,
            unknown_short,
            name,
            description,
            unknown_data,
        }) => {
            out.write(&id.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&character_restriction.to_le_bytes()).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, flags).map_err(SerializeError::WriteError)?;
            out.write(&r#type.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&subtype.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&element.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&slash.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&thrust.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&pierce.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&strike.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_byte.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&target_type.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&range.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&area.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_1.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_2.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_3.to_le_bytes()).map_err(SerializeError::WriteError)?;
            for eff in effects {
                out.write(&eff.id.to_le_bytes()).map_err(SerializeError::WriteError)?;
                for x in eff.data {
                    out.write(&x.to_le_bytes()).map_err(SerializeError::WriteError)?;
                }
            }
            out.write(&str.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&def.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&ats.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&adf.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&acc.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&eva.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&spd.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&mov.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&hp.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&ep.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&price.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&carry_limit.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&sort_id.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_short.to_le_bytes()).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, name).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, description).map_err(SerializeError::WriteError)?;
            for x in unknown_data {
                out.write(&x.to_le_bytes()).map_err(SerializeError::WriteError)?;
            }
        }
        Entry::Magic(entry_data::Magic {
            id,
            character_restriction,
            flags,
            category,
            r#type,
            element,
            juna_specific,
            target_type,
            target_range,
            target_size,
            unknown_float_1,
            unknown_float_2,
            unknown_float_3,
            effects,
            cast_delay,
            recovery_delay,
            cost,
            unbalance_bonus,
            break_bonus,
            level_learn,
            sort_id,
            animation,
            name,
            description,
        }) => {
            out.write(&id.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&character_restriction.to_le_bytes()).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, flags).map_err(SerializeError::WriteError)?;
            out.write(&category.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&r#type.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&element.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&juna_specific.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&target_type.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&target_range.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&target_size.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_1.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_2.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_3.to_le_bytes()).map_err(SerializeError::WriteError)?;
            for eff in effects {
                out.write(&eff.id.to_le_bytes()).map_err(SerializeError::WriteError)?;
                for x in eff.data {
                    out.write(&x.to_le_bytes()).map_err(SerializeError::WriteError)?;
                }
            }
            out.write(&cast_delay.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&recovery_delay.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&cost.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unbalance_bonus.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&break_bonus.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&level_learn.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&sort_id.to_le_bytes()).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, animation).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, name).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, description).map_err(SerializeError::WriteError)?;
        }
        Entry::MagicBo(entry_data::MagicBo { id, effects, turns }) => {
            out.write(&id.to_le_bytes()).map_err(SerializeError::WriteError)?;
            for eff in effects {
                out.write(&eff.id.to_le_bytes()).map_err(SerializeError::WriteError)?;
                for x in eff.data {
                    out.write(&x.to_le_bytes()).map_err(SerializeError::WriteError)?;
                }
            }
            out.write(&turns.to_le_bytes()).map_err(SerializeError::WriteError)?;
        }
        Entry::SlotCost(data) => {
            data.serialize(out).map_err(SerializeError::WriteError)?;
        }
        Entry::Status(entry_data::Status {
            script,
            texture,
            model,
            model_scale,
            camera_pivot_height,
            hitbox_size,
            unknown_float_1,
            unknown_float_2,
            unknown_float_3,
            unknown_float_4,
            unknown_short_1,
            unknown_short_2,
            is_female,
            level,
            hp_base,
            hp_growth,
            max_ep,
            start_ep,
            max_cp,
            start_cp,
            str_base,
            str_growth,
            def_base,
            def_growth,
            ats_base,
            ats_growth,
            adf_base,
            adf_growth,
            dex_base,
            dex_growth,
            agl_base,
            agl_growth,
            eva,
            spd_base,
            spd_growth,
            mov_base,
            mov_growth,
            exp_base,
            exp_growth,
            brk_mult_base,
            brk_mult_growth,
            earth_efficacy,
            water_efficacy,
            fire_efficacy,
            wind_efficacy,
            time_efficacy,
            space_efficacy,
            mirage_efficacy,
            psn_efficacy,
            seal_efficacy,
            mute_efficacy,
            blnd_efficacy,
            slp_efficacy,
            burn_efficacy,
            frz_efficacy,
            petr_efficacy,
            fnt_efficacy,
            conf_efficacy,
            chrm_efficacy,
            dblw_efficacy,
            nmr_efficacy,
            dlay_efficacy,
            vnsh_efficacy,
            s_dwn_efficacy,
            slash_efficacy,
            thrust_efficacy,
            pierce_efficacy,
            strike_efficacy,
            earth_sepith_base,
            water_sepith_base,
            fire_sepith_base,
            wind_sepith_base,
            time_sepith_base,
            space_sepith_base,
            mirage_sepith_base,
            sepith_mass_base,
            earth_sepith_growth,
            water_sepith_growth,
            fire_sepith_growth,
            wind_sepith_growth,
            time_sepith_growth,
            space_sepith_growth,
            mirage_sepith_growth,
            sepith_mass_growth,
            drop_id_1,
            drop_chance_1,
            drop_id_2,
            drop_chance_2,
            unknown_float_5,
            unknown_float_6,
            flags,
            name,
            description,
        }) => {
            serialize_utf8cstring(out, script).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, texture).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, model).map_err(SerializeError::WriteError)?;
            out.write(&model_scale.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&camera_pivot_height.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&hitbox_size.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_1.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_2.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_3.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_4.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_short_1.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_short_2.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&is_female.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&level.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&hp_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&hp_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&max_ep.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&start_ep.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&max_cp.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&start_cp.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&str_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&str_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&def_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&def_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&ats_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&ats_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&adf_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&adf_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&dex_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&dex_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&agl_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&agl_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&eva.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&spd_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&spd_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&mov_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&mov_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&exp_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&exp_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&brk_mult_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&brk_mult_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&earth_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&water_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&fire_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&wind_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&time_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&space_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&mirage_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&psn_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&seal_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&mute_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&blnd_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&slp_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&burn_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&frz_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&petr_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&fnt_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&conf_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&chrm_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&dblw_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&nmr_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&dlay_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&vnsh_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&s_dwn_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&slash_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&thrust_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&pierce_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&strike_efficacy.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&earth_sepith_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&water_sepith_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&fire_sepith_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&wind_sepith_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&time_sepith_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&space_sepith_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&mirage_sepith_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&sepith_mass_base.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&earth_sepith_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&water_sepith_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&fire_sepith_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&wind_sepith_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&time_sepith_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&space_sepith_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&mirage_sepith_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&sepith_mass_growth.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&drop_id_1.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&drop_chance_1.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&drop_id_2.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&drop_chance_2.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_5.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float_6.to_le_bytes()).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, flags).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, name).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, description).map_err(SerializeError::WriteError)?;
        }
        Entry::Voice(entry_data::Voice {
            unknown_short_1,
            unknown_byte,
            clip,
            unknown_short_2,
            unknown_float,
            unknown_data,
        }) => {
            out.write(&unknown_short_1.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_byte.to_le_bytes()).map_err(SerializeError::WriteError)?;
            serialize_utf8cstring(out, clip).map_err(SerializeError::WriteError)?;
            out.write(&unknown_short_2.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(&unknown_float.to_le_bytes()).map_err(SerializeError::WriteError)?;
            out.write(unknown_data).map_err(SerializeError::WriteError)?;
        }
    }
    Ok(())
}

fn serialized_data_length(entry: &Entry) -> Option<u16> {
    match entry {
        Entry::Generic(entry_data::Generic { data, .. }) => data.len().try_into().ok(),
        Entry::CharRevise(entry_data::CharRevise { char, .. }) => char.len().try_into().ok().and_then(|n: u16| n.checked_add(15)),
        Entry::CompHelpData(entry_data::CompHelpData { text, .. }) => text.len().try_into().ok().and_then(|n: u16| n.checked_add(7)),
        Entry::Item(entry_data::Item { flags, name, description, .. }) => [flags.len(), name.len(), description.len()].iter().fold(Some(142), |sum, n| sum?.checked_add((*n).try_into().ok()?)),
        Entry::Magic(entry_data::Magic {
            flags, animation, name, description, ..
        }) => [flags.len(), animation.len(), name.len(), description.len()]
            .iter()
            .fold(Some(110), |sum, n| sum?.checked_add((*n).try_into().ok()?)),
        Entry::MagicBo(_) => Some(74),
        Entry::SlotCost(_) => Some(18),
        Entry::Status(entry_data::Status {
            script,
            texture,
            model,
            flags,
            name,
            description,
            ..
        }) => [script.len(), texture.len(), model.len(), flags.len(), name.len(), description.len()]
            .iter()
            .fold(Some(203), |sum, n| sum?.checked_add((*n).try_into().ok()?)),
        Entry::Voice(entry_data::Voice { clip, unknown_data, .. }) => [clip.len(), unknown_data.len()].iter().fold(Some(10), |sum, n| sum?.checked_add((*n).try_into().ok()?)),
    }
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
    fn test() {
        todo!("test each individual table header type?");
    }

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
