use super::{ActionTableEntry, AlDat, AlDatEntry, AlgoTableEntry};
use crate::io::MemWriter;

pub fn serialize(aldat: &AlDat) -> Option<std::vec::Vec<u8>> {
    let mut writer = MemWriter::with_capacity(1 << 16);

    // magic header or something like that
    writer.write_data(&[0x20, 0, 0, 0, 0x20, 0, 0, 0]);

    // offset of the table list - the name is padded such that it is a multiple of 4
    let padding_before_table_list = {
        // for no discernible reason, the padding is at least 3 and the last pad byte is 0xff
        let n = aldat.name.len_with_padding().get() + 1; // + 1 for the 0xff byte which is there for some reason???
        let mut p = 3;
        // TODO: make this better
        while ((n + p) % 4) != 0 {
            p += 1;
        }
        p
    };

    let table_list_offset: usize = 33 + aldat.name.len_with_padding().get() + padding_before_table_list;
    writer.write_u32_le(table_list_offset.try_into().unwrap());

    // redundant stuff
    let n1 = 4 * aldat.entries.len();
    writer.write_u32_le(n1.try_into().unwrap());
    writer.write_u32_le((table_list_offset + n1).try_into().unwrap());

    // number of tables
    writer.write_u32_le(aldat.entries.len().try_into().unwrap());

    // calculate table data offsets and name offsets
    let table_names_offset: usize = table_list_offset + 6 * aldat.entries.len();
    let table_names_end_offset = table_names_offset + aldat.entries.iter().map(|ent| ent.name_len_with_padding()).sum::<usize>();
    let table_data_padding = if table_names_end_offset % 4 == 0 { 0 } else { 4 - table_names_end_offset % 4 };
    let table_data_offset = table_names_end_offset + table_data_padding;
    writer.write_u32_le(table_names_end_offset.try_into().unwrap());

    // unknown
    writer.write_data(aldat.unk2.as_slice());

    // script name
    writer.write_padded_ascii_str(aldat.name.as_padded_ascii_str());

    // padding
    if padding_before_table_list != 0 {
        let buf = [0u8; 8];
        writer.write_data(&buf[..padding_before_table_list]);
    }
    writer.write_u8(0xff);

    {
        // table data offsets
        let mut off = table_data_offset;
        for (i, entry) in aldat.entries.iter().enumerate() {
            writer.write_u32_le(off.try_into().ok()?);
            off += entry.data_len();

            if let Some(next_entry) = aldat.entries.get(i + 1) {
                let pad_target = aldat.alignment.get(&next_entry.name().to_owned()).copied()?; // TODO: datastructure that doesn't require cloning
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
        for ent in aldat.entries.iter() {
            writer.write_u16_le(off.try_into().ok()?);
            off += ent.name_len_with_padding();
        }
    }

    // table names
    for ent in aldat.entries.iter() {
        writer.write_padded_ascii_str(ent.name());
    }

    // pad start of data blocks to multiple of 4
    {
        let buf = [0u8; 4];
        writer.write_data(&buf[..table_data_padding]);
    }

    // write table data
    for (i, entry) in aldat.entries.iter().enumerate() {
        entry.serialize_data(&mut writer);
        // padding
        if let Some(next_entry) = aldat.entries.get(i + 1) {
            let pad_target = aldat.alignment.get(&next_entry.name().to_owned()).copied()?; // TODO: datastructure that doesn't require copying
            let n = writer.len();
            let padding_len = pad_target - n % pad_target;
            assert!(padding_len > 0);
            writer.write_u8(1);
            for i in 1..padding_len {
                writer.write_u8(0);
            }
        } else {
            // last entry
            writer.write_u8(1);
        }
    }

    // TODO: final padding
    if aldat.entries.is_empty() {}

    Some(writer.into_inner())
}

impl AlDatEntry {
    fn serialize_data(&self, writer: &mut MemWriter) -> Option<()> {
        match self {
            AlDatEntry::ActionTable(entries) => {
                for entry in entries {
                    entry.serialize_data(writer)?;
                }
            }
            AlDatEntry::AlgoTable(entries) => {
                for entry in entries {
                    entry.serialize_data(writer)?;
                }
            }
            AlDatEntry::Generic { name: _, data } => {
                writer.write_data(data.as_slice());
            }
        }
        Some(())
    }
}

impl ActionTableEntry {
    fn serialize_data(&self, vec: &mut MemWriter) -> Option<()> {
        todo!()
    }
}

impl AlgoTableEntry {
    fn serialize_data(&self, vec: &mut MemWriter) -> Option<()> {
        todo!()
    }
}

impl AlDatEntry {
    fn data_len(&self) -> usize {
        match self {
            AlDatEntry::ActionTable { .. } => 212,
            AlDatEntry::AlgoTable { .. } => 32,
            AlDatEntry::Generic { name: _, data } => data.len(),
        }
    }
}
