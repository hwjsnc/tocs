/*
For a description of the pkg file structure, see https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/blob/master/docs/PKG-File-Structure.md
*/

use smallvec::SmallVec;
use smol_str::SmolStr;

fn read_byte<F: std::io::Read>(f: &mut F) -> Result<u8, std::io::Error> {
    let a: [u8; 1] = read_bytes(f)?;
    Ok(a[0])
}

fn read_bytes<const N: usize, F: std::io::Read>(f: &mut F) -> Result<[u8; N], std::io::Error> {
    let mut buf: [u8; N] = [0; N];
    f.read_exact(&mut buf)?;
    //Ok(buf.try_into().expect("length should match"))
    Ok(buf)
}

pub fn load_filelist<F: std::io::Read>(f: &mut F) -> Result<std::vec::Vec<FileListEntry>, std::io::Error> {
    let data = read_bytes::<4, _>(f)?;
    if data != [0, 0, 0, 0] {
        log::error!("Expected pkg to start with four 0 bytes");
        todo!();
    }
    let number_of_files = u32::from_le_bytes(read_bytes::<4, _>(f)?);
    log::debug!("Expecting {number_of_files} files");

    let mut filelist = std::vec::Vec::<FileListEntry>::new();
    for i in 0..number_of_files {
        let entry = parse_filelist_entry(f).unwrap(); // TODO
        let FileListEntry {
            filename,
            raw_size,
            compressed_size,
            data_offset,
            compression,
        } = &entry;
        log::debug!(
            "filelist entry {} of {}: name = {filename}, raw size = {raw_size}, compressed size = {compressed_size}, data offset = {data_offset} (0x{data_offset:x}), compression = {compression:?}",
            i + 1,
            number_of_files
        );
        filelist.push(entry);
    }

    // TODO: check that data ranges are overlap-free and leave no space?

    Ok(filelist)
}

#[derive(Clone, Copy, Debug)]
pub enum Compression {
    None,
    /*
    LZSS,
    LZ4,
    Zstandard,
    */
    Unknown(u32),
}

impl std::convert::From<u32> for Compression {
    fn from(x: u32) -> Compression {
        match x {
            1 => Compression::None,
            _ => Compression::Unknown(x),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileListEntry {
    filename: SmolStr,
    raw_size: u32,
    compressed_size: u32,
    data_offset: u32,
    compression: Compression,
}

pub fn parse_cutf8_padded<F: std::io::Read>(f: &mut F, chunk_size: usize) -> Result<SmolStr, ()> {
    // TODO error type
    // read raw bytes until we hit 0
    let mut data: SmallVec<[u8; 64]> = SmallVec::new();
    let mut i = 0;
    loop {
        let x = read_byte(f).unwrap(); // TODO
        if x == 0 {
            break;
        } else {
            data.push(x);
        }
        i += 1;
    }

    // read padding until we hit the chunk size
    while i % chunk_size != 0 {
        if read_byte(f).unwrap() != 0 {
            // TODO
            panic!("expected padding with zero bytes");
        }
        i += 1;
    }

    // check utf-8
    match std::str::from_utf8(&data) {
        Ok(s) => Ok(SmolStr::from(s)),
        _ => todo!(""),
    }
}

fn parse_filelist_entry<F: std::io::Read>(f: &mut F) -> Result<FileListEntry, ()> {
    // TODO error type
    // TODO: errors
    let filename = parse_cutf8_padded(f, 64).unwrap();
    let raw_size = u32::from_le_bytes(read_bytes::<4, _>(f).unwrap());
    let compressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).unwrap());
    let data_offset = u32::from_le_bytes(read_bytes::<4, _>(f).unwrap());
    let compression = Compression::from(u32::from_le_bytes(read_bytes::<4, _>(f).unwrap()));
    Ok(FileListEntry {
        filename,
        raw_size,
        compressed_size,
        data_offset,
        compression,
    })
}
