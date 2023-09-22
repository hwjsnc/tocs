use super::{ActionTableEntry, AlgoTableEntry, Effect, Script, ScriptEntry, ScriptNamePadding, SummonTableEntry};

use crate::io::MemReader;

/// Checks the given condition and returns `None` if it is false.
macro_rules! ensure {
    ($cond:expr) => {
        if !($cond) {
            return None;
        }
    };
}

fn remove_end_padding_from_entry_data(padded: &[u8]) -> Option<&[u8]> {
    // padding is a 1 byte followed by some number of zero bytes; we check this from the back
    for (i, x) in padded.iter().copied().enumerate().rev() {
        match x {
            0 => {}                         // still padding
            1 => return Some(&padded[..i]), // start of padding
            _ => return None,               // invalid padding
        }
    }
    None
}

pub fn parse(data: &[u8]) -> Option<Script> {
    let mut reader = MemReader::new(data);

    // check magic bytes
    let a = reader.read_u32_le()?;
    let b = reader.read_u32_le()?;

    ensure!(a == 0x20);
    ensure!(b == 0x20);

    // number of tables and a bunch of useless stuff
    let table_list_offset = reader.read_u32_le()?;
    let n1 = reader.read_u32_le()?;
    let n2 = reader.read_u32_le()?;
    let n_tables_u32 = reader.read_u32_le()?;
    let n_tables: usize = n_tables_u32.try_into().unwrap();
    let table_names_end_offset = reader.read_u32_le()?;

    ensure!(n1 == 4 * n_tables_u32);
    ensure!(n2 == table_list_offset + n1);

    // read another magic value
    let n3 = reader.read_u32_le()?;
    ensure!(n3 == 0xabcdef00);

    let script_name = reader.read_c_ascii_str().ok()?;

    // check padding between end of script name and start of table name offsets
    let name_padding = {
        let script_padding = data.get(reader.offset()..usize::try_from(table_list_offset).unwrap())?;
        // count 0x00 bytes, 0xff bytes, other bytes
        let mut zero = 0;
        let mut ff = 0;
        let mut other = 0;
        for x in script_padding {
            match *x {
                0 => zero += 1,
                0xff => ff += 1,
                _ => other += 1,
            }
        }
        // we expect one of three patterns:
        // - null bytes, followed by 0xff (the most common case; there are at least 2 null bytes but we don't check that)
        // - null bytes, followed by 0xff, followed by [0x00, 0x00, 0x00, 0xff]
        // - no padding at all
        ensure!(other == 0);
        if ff == 0 && zero == 0 {
            ScriptNamePadding::None
        } else if ff == 1 {
            ScriptNamePadding::Once
        } else if ff == 2 && zero >= 3 {
            ScriptNamePadding::Twice
        } else {
            return None;
        }
    };

    // read offsets of table names and data
    reader.set_offset(usize::try_from(table_list_offset).unwrap());

    let mut table_name_offsets = std::vec::Vec::with_capacity(n_tables);
    let mut table_data_offsets = std::vec::Vec::with_capacity(n_tables);

    for _ in 0..n_tables {
        let off = reader.read_u32_le()?;
        ensure!(off % 4 == 0);
        table_data_offsets.push(off);
    }
    for _ in 0..n_tables {
        table_name_offsets.push(reader.read_u16_le()?);
    }

    // read table names and contents, record alignments
    let mut tables = std::vec::Vec::with_capacity(n_tables);
    let mut alignment = std::collections::BTreeMap::new();
    for i in 0..n_tables {
        let name = reader.clone_at_offset(table_name_offsets[i].try_into().unwrap()).read_c_ascii_str().ok()?;

        // we assume the table data is always sorted. it seems to work out.
        let data_start = usize::try_from(table_data_offsets[i]).unwrap();
        let data_end = table_data_offsets.get(i + 1).copied().map(|n| usize::try_from(n).unwrap()).unwrap_or(data.len());
        let table_data_including_terminator = data.get(data_start..data_end)?;

        let table_data = remove_end_padding_from_entry_data(table_data_including_terminator)?;

        let table = match name.to_str() {
            "ActionTable" => ScriptEntry::ActionTable(read_action_table_entries(table_data)?),
            "AlgoTable" => ScriptEntry::AlgoTable(read_algo_table_entries(table_data)?),
            "SummonTable" => ScriptEntry::SummonTable(read_summon_table_entries(table_data)?),
            _ => ScriptEntry::Generic {
                name: name.to_owned(),
                data: table_data.to_owned(),
            },
        };

        let align = largest_power_of_two_that_divides(data_start);
        alignment.entry(name.to_owned()).and_modify(|a| *a = std::cmp::min(*a, align)).or_insert(align);

        tables.push(table);
    }

    // check that entry names are densely packed
    if let Some(start_offset) = table_name_offsets.first().copied() {
        let combined_length_of_entry_names = tables.iter().map(|ent| ent.name_len_with_padding()).sum::<usize>();
        ensure!(table_names_end_offset == start_offset as u32 + combined_length_of_entry_names as u32);
    }

    Some(Script {
        alignment,
        name: script_name.to_owned(),
        name_padding,
        entries: tables,
    })
}

fn largest_power_of_two_that_divides(n: usize) -> usize {
    1 << n.trailing_zeros()
}

pub fn read_action_table_entries(data: &[u8]) -> Option<std::vec::Vec<ActionTableEntry>> {
    let mut reader = MemReader::new(data);
    let mut entries = std::vec::Vec::with_capacity(data.len() / 212);

    while !reader.at_end() {
        let id = reader.read_i16_le()?;
        let category = reader.read_u16_le()?;

        let target_type = reader.read_u8()?;
        let target_range = reader.read_u8()?;
        let target_size = reader.read_u16_le()?;

        let unk4 = reader.read_f32_le()?;
        let unk5 = reader.read_f32_le()?;
        let unk6 = reader.read_f32_le()?;

        let cast_delay = reader.read_u16_le()?;
        let recovery_delay = reader.read_u16_le()?;

        let mut effect_ids = <[u16; 5]>::default();
        let mut all_parameters = <[i32; 15]>::default();
        for id in effect_ids.iter_mut() {
            *id = reader.read_u16_le()?;
        }
        let unk10 = reader.read_u16_le()?;
        for param in all_parameters.iter_mut() {
            *param = reader.read_i32_le()?;
        }

        let effects = [
            Effect {
                id: effect_ids[0],
                parameters: [all_parameters[0], all_parameters[1], all_parameters[2]],
            },
            Effect {
                id: effect_ids[1],
                parameters: [all_parameters[3], all_parameters[4], all_parameters[5]],
            },
            Effect {
                id: effect_ids[2],
                parameters: [all_parameters[6], all_parameters[7], all_parameters[8]],
            },
            Effect {
                id: effect_ids[3],
                parameters: [all_parameters[9], all_parameters[10], all_parameters[11]],
            },
            Effect {
                id: effect_ids[4],
                parameters: [all_parameters[12], all_parameters[13], all_parameters[14]],
            },
        ];

        let unk18 = reader.read_u32_le()?;

        let flags = reader.read_padded_ascii_str::<16>().ok()?.to_owned();
        let animation = reader.read_padded_utf8_str::<32>().ok()?.to_owned();
        let name = reader.read_padded_utf8_str::<64>().ok()?.to_owned();

        entries.push(ActionTableEntry {
            id,
            category,
            target_type,
            target_range,
            target_size,
            unk4,
            unk5,
            unk6,
            cast_delay,
            recovery_delay,
            effects,
            unk10,
            unk18,
            flags,
            animation,
            name,
        });
    }

    Some(entries)
}

pub fn read_algo_table_entries(data: &[u8]) -> Option<std::vec::Vec<AlgoTableEntry>> {
    let mut reader = MemReader::new(data);
    let mut entries = std::vec::Vec::with_capacity(data.len() / 32);

    while !reader.at_end() {
        let action_or_magic_id = reader.read_i16_le()?;
        let condition = reader.read_byte()?;
        let chance = reader.read_byte()?;
        let use_limit = reader.read_byte()?;
        let target_priority = reader.read_byte()?;
        let unk_byte_1 = reader.read_byte()?;
        let unk_byte_2 = reader.read_byte()?;
        let cond_param_1 = reader.read_u32_le()?;
        let cond_param_2 = reader.read_u32_le()?;
        let cond_param_3 = reader.read_u32_le()?;
        let target_param = reader.read_u32_le()?;
        let use_limit_after_retry = reader.read_u32_le()?;
        let unk6 = reader.read_u32_le()?;

        entries.push(AlgoTableEntry {
            action_or_magic_id,
            condition,
            chance,
            use_limit,
            target_priority,
            unk_byte_1,
            unk_byte_2,
            cond_param_1,
            cond_param_2,
            cond_param_3,
            target_param,
            use_limit_after_retry,
            unk6,
        });
    }

    Some(entries)
}

pub fn read_summon_table_entries(data: &[u8]) -> Option<std::vec::Vec<SummonTableEntry>> {
    let mut reader = MemReader::new(data);
    let mut entries = std::vec::Vec::with_capacity(data.len() / 32);

    while !reader.at_end() {
        let unk1 = reader.read_u16_le()?;
        let unk2 = reader.read_u16_le()?;
        let monster = reader.read_padded_ascii_str::<32>().ok()?.to_owned();

        entries.push(SummonTableEntry { unk1, unk2, monster });
    }

    Some(entries)
}
