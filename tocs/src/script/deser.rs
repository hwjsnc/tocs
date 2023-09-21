use super::{ActionTableEntry, AlgoTableEntry, Script, ScriptEntry, ScriptNamePadding};

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

    // read more
    let unk2 = reader.read_chunk::<4>()?;
    let script_name = reader.read_c_ascii_str().ok()?;

    println!("offset after script name = 0x{:x}, table list offset = 0x{:?}", reader.offset(), table_list_offset);

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

    println!("before data offsets 0x{:x}", reader.offset());
    for _ in 0..n_tables {
        let off = reader.read_u32_le()?;
        ensure!(off % 4 == 0);
        table_data_offsets.push(off);
    }
    println!("after data offsets 0x{:x}", reader.offset());
    for _ in 0..n_tables {
        table_name_offsets.push(reader.read_u16_le()?);
    }

    println!("after name offsets 0x{:x}", reader.offset());

    // read table names and contents, record alignments
    let mut tables = std::vec::Vec::with_capacity(n_tables);
    let mut alignment = std::collections::BTreeMap::new();
    for i in 0..n_tables {
        println!("start {i} (name offset 0x{:x})", table_name_offsets[i]);
        let name = reader.clone_at_offset(table_name_offsets[i].try_into().unwrap()).read_c_ascii_str().ok()?;

        // we assume the table data is always sorted. it seems to work out.
        let data_start = usize::try_from(table_data_offsets[i]).unwrap();
        let data_end = table_data_offsets.get(i + 1).copied().map(|n| usize::try_from(n).unwrap()).unwrap_or(data.len());
        let table_data_including_terminator = data.get(data_start..data_end)?;

        println!(
            "data {}; next one: {}, offset 0x{:x}",
            name.to_str(),
            reader
                .clone_at_offset(table_name_offsets[i].try_into().unwrap())
                .read_c_ascii_str()
                .ok()
                .map(|s| s.to_str())
                .unwrap_or(""),
            table_data_offsets.get(i + 1).map(|x| *x as usize).unwrap_or(data.len())
        );
        let table_data = remove_end_padding_from_entry_data(table_data_including_terminator)?;
        //println!("foo {i}");

        let table = match name.to_str() {
            /*
            "ActionTable" => ScriptEntry::ActionTable(read_action_table_entries(table_data)?),
            "AlgoTable" => ScriptEntry::ActionTable(read_action_table_entries(table_data)?),
            */
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
        unk2,
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

    loop {
        let id = reader.read_i16_le()?;
        if id == 1 {
            ensure!(reader.rest() == &[0, 0]);
            break;
        }

        let unk1 = reader.read_u16_le()?;
        let unk2 = reader.read_chunk::<4>()?;
        let unk3 = reader.read_f32_le()?;
        let unk4 = reader.read_f32_le()?;
        let unk5 = reader.read_f32_le()?;

        let unk10 = reader.read_chunk::<4>()?;

        let unk11 = reader.read_u16_le()?;
        let unk12 = reader.read_u16_le()?;
        let unk13 = reader.read_u16_le()?;
        let unk14 = reader.read_u16_le()?;
        let unk15 = reader.read_u16_le()?;
        let unk16 = reader.read_u16_le()?;

        let unk17: [i32; 15] = [
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
        ];

        let unk18 = reader.read_u16_le()?;
        let unk19 = reader.read_u16_le()?;

        let flags = reader.read_padded_ascii_str::<16>().ok()?.to_owned();
        let animation = reader.read_padded_utf8_str::<32>().ok()?.to_owned();
        let name = reader.read_padded_utf8_str::<64>().ok()?.to_owned();

        entries.push(ActionTableEntry {
            id,
            unk1,
            unk2,
            unk3,
            unk4,
            unk5,
            unk10,
            unk11,
            unk12,
            unk13,
            unk14,
            unk15,
            unk16,
            unk17,
            unk18,
            unk19,
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

    while reader.offset() + 32 < data.len() {
        let action_or_magic_id = reader.read_i16_le()?;
        let condition = reader.read_byte()?;
        let chance = reader.read_byte()?;
        let use_limit = reader.read_byte()?;
        let target_type = reader.read_byte()?;
        let unk_byte_1 = reader.read_byte()?;
        let unk_byte_2 = reader.read_byte()?;
        let mut params = [0u32; 6];
        for x in params.iter_mut() {
            *x = reader.read_u32_le()?;
        }

        entries.push(AlgoTableEntry {
            action_or_magic_id,
            condition,
            chance,
            use_limit,
            target_type,
            unk_byte_1,
            unk_byte_2,
            params,
        });
    }

    // check final padding
    ensure!(reader.read_byte()? == 1);
    ensure!(reader.rest().iter().copied().all(|b| b == 0));

    Some(entries)
}
