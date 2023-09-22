use super::{ActionTableEntry, AlgoTableEntry, Script, ScriptEntry, ScriptNamePadding, SummonTableEntry};
use crate::io::MemWriter;

pub fn serialize(script: &Script) -> Option<std::vec::Vec<u8>> {
    let mut writer = MemWriter::with_capacity(1 << 16);

    // magic header or something like that
    writer.write_data(&[0x20, 0, 0, 0, 0x20, 0, 0, 0]);

    // offset of the table list - the name is padded such that it is a multiple of 4
    // Note: this value is only relevant if script.name_padding != None
    let padding_before_table_list = {
        // for no discernible reason, the padding is at least 3 and the last pad byte is 0xff
        let n = script.name.len_with_padding().get() + 1; // + 1 for the 0xff byte which is there for some reason???
        let mut p = 3;
        // TODO: make this better
        while ((n + p) % 4) != 0 {
            p += 1;
        }
        p
    };

    let table_list_offset: usize = match script.name_padding {
        ScriptNamePadding::None => 32 + script.name.len_with_padding().get(),
        ScriptNamePadding::Once => 33 + script.name.len_with_padding().get() + padding_before_table_list,
        ScriptNamePadding::Twice => 37 + script.name.len_with_padding().get() + padding_before_table_list,
    };
    writer.write_u32_le(table_list_offset.try_into().unwrap());

    // redundant stuff
    let n1 = 4 * script.entries.len();
    writer.write_u32_le(n1.try_into().unwrap());
    writer.write_u32_le((table_list_offset + n1).try_into().unwrap());

    // number of tables
    writer.write_u32_le(script.entries.len().try_into().unwrap());

    // calculate table data offsets and name offsets
    let table_names_offset: usize = table_list_offset + 6 * script.entries.len();
    let table_names_end_offset = table_names_offset + script.entries.iter().map(|ent| ent.name_len_with_padding()).sum::<usize>();
    let table_data_padding = if table_names_end_offset % 4 == 0 { 0 } else { 4 - table_names_end_offset % 4 };
    let table_data_offset = table_names_end_offset + table_data_padding;
    writer.write_u32_le(table_names_end_offset.try_into().unwrap());

    // magic value
    writer.write_u32_le(0xabcdef00);

    // script name
    writer.write_padded_ascii_str(script.name.as_padded_ascii_str());

    // padding
    if script.name_padding != ScriptNamePadding::None {
        if padding_before_table_list != 0 {
            let buf = [0u8; 8];
            writer.write_data(&buf[..padding_before_table_list]);
        }
        writer.write_u8(0xff);
    }
    if script.name_padding == ScriptNamePadding::Twice {
        writer.write_data([0, 0, 0, 0xff].as_slice());
    }

    assert_eq!(writer.len(), table_list_offset);

    {
        // table data offsets
        let mut off = table_data_offset;
        for (i, entry) in script.entries.iter().enumerate() {
            writer.write_u32_le(off.try_into().ok()?);
            off += entry.data_len();

            if let Some(next_entry) = script.entries.get(i + 1) {
                let pad_target = script.alignment.get(&next_entry.name().to_owned()).copied()?; // TODO: datastructure that doesn't require cloning
                let padding_len = if off % pad_target == 0 { pad_target } else { pad_target - off % pad_target };
                off += padding_len;
            } else {
                off += 1
            }
        }
    }

    {
        // table name offsets
        let mut off = table_names_offset;
        for ent in script.entries.iter() {
            writer.write_u16_le(off.try_into().ok()?);
            off += ent.name_len_with_padding();
        }
    }

    // table names
    for ent in script.entries.iter() {
        writer.write_padded_ascii_str(ent.name());
    }

    // pad start of data blocks to multiple of 4
    {
        let buf = [0u8; 4];
        writer.write_data(&buf[..table_data_padding]);
    }

    // write table data
    for (i, entry) in script.entries.iter().enumerate() {
        entry.serialize_data(&mut writer);
        // padding
        if let Some(next_entry) = script.entries.get(i + 1) {
            let pad_target = script.alignment.get(&next_entry.name().to_owned()).copied()?; // TODO: datastructure that doesn't require copying
            let n = writer.len();
            let padding_len = pad_target - n % pad_target;
            assert!(padding_len > 0);
            writer.write_u8(1);
            writer.write_zeros(padding_len - 1);
        } else {
            // last entry
            writer.write_u8(1);
        }
    }

    Some(writer.into_inner())
}

impl ScriptEntry {
    fn serialize_data(&self, writer: &mut MemWriter) -> Option<()> {
        match self {
            ScriptEntry::ActionTable(entries) => entries.iter().try_for_each(|entry| entry.serialize_data(writer))?,
            ScriptEntry::AlgoTable(entries) => entries.iter().try_for_each(|entry| entry.serialize_data(writer))?,
            ScriptEntry::SummonTable(entries) => entries.iter().try_for_each(|entry| entry.serialize_data(writer))?,
            ScriptEntry::Generic { name: _, data } => {
                writer.write_data(data.as_slice());
            }
        }
        Some(())
    }
}

impl ActionTableEntry {
    fn serialize_data(&self, writer: &mut MemWriter) -> Option<()> {
        let Self {
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
        } = self;
        writer.write_i16_le(*id);
        writer.write_u16_le(*category);
        writer.write_u8(*target_type);
        writer.write_u8(*target_range);
        writer.write_u16_le(*target_size);
        writer.write_f32_le(*unk4);
        writer.write_f32_le(*unk5);
        writer.write_f32_le(*unk6);
        writer.write_u16_le(*cast_delay);
        writer.write_u16_le(*recovery_delay);
        for eff in effects.iter() {
            writer.write_u16_le(eff.id);
        }
        writer.write_u16_le(*unk10);
        for eff in effects.iter() {
            for param in eff.parameters {
                writer.write_i32_le(param);
            }
        }
        writer.write_u32_le(*unk18);
        writer.write_padded_ascii_str(flags.as_padded_ascii_str());
        writer.write_padded_utf8_str(animation.as_padded_utf8_str());
        writer.write_padded_utf8_str(name.as_padded_utf8_str());

        Some(())
    }
}

impl AlgoTableEntry {
    fn serialize_data(&self, writer: &mut MemWriter) -> Option<()> {
        let Self {
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
        } = self;
        writer.write_i16_le(*action_or_magic_id);
        writer.write_u8(*condition);
        writer.write_u8(*chance);
        writer.write_u8(*use_limit);
        writer.write_u8(*target_priority);
        writer.write_u8(*unk_byte_1);
        writer.write_u8(*unk_byte_2);
        writer.write_u32_le(*cond_param_1);
        writer.write_u32_le(*cond_param_2);
        writer.write_u32_le(*cond_param_3);
        writer.write_u32_le(*target_param);
        writer.write_u32_le(*use_limit_after_retry);
        writer.write_u32_le(*unk6);
        Some(())
    }
}

impl SummonTableEntry {
    fn serialize_data(&self, writer: &mut MemWriter) -> Option<()> {
        let Self { unk1, unk2, monster } = self;
        writer.write_u16_le(*unk1);
        writer.write_u16_le(*unk2);
        writer.write_padded_ascii_str(monster.as_padded_ascii_str());
        Some(())
    }
}

impl ScriptEntry {
    fn data_len(&self) -> usize {
        match self {
            ScriptEntry::ActionTable(entries) => 212 * entries.len(),
            ScriptEntry::AlgoTable(entries) => 32 * entries.len(),
            ScriptEntry::SummonTable(entries) => 36 * entries.len(),
            ScriptEntry::Generic { name: _, data } => data.len(),
        }
    }
}
