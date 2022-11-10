/*
For a description of the pkg file structure, see https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/blob/master/docs/PKG-File-Structure.md
*/

// TODO: actually use this, haha
const MAX_DATA_SIZE: usize = 1 << 30; // a single asset > 1GB is pretty unlikely

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

pub fn load_filelist<F: std::io::Read>(f: &mut F) -> Result<std::vec::Vec<FileListEntry>, PkgError> {
    let data = read_bytes::<4, _>(f).map_err(PkgError::IOError)?; // magic header or a hash or something???
    let number_of_files = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
    log::debug!("Expecting {number_of_files} files");

    let mut filelist = std::vec::Vec::<FileListEntry>::new();
    for i in 0..number_of_files {
        log::debug!("parsing filelist entry {} of {}", i + 1, number_of_files);
        let entry = parse_filelist_entry(f)?;
        filelist.push(entry);
    }

    // TODO: check that data ranges are overlap-free and leave no space?

    Ok(filelist)
}

#[derive(Clone, Copy, Debug)]
pub enum Compression {
    None,
    LZSS,
    LZ4,
    //Zstd?,
    Unknown(u32),
}

#[derive(Debug)]
pub enum PkgError {
    IOError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    FilelistError(String),
    DecompressionError(String),
    CompressionError(String),
}

impl std::fmt::Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Compression::None => write!(f, "uncompressed"),
            Compression::LZSS => write!(f, "LZSS"),
            Compression::LZ4 => write!(f, "LZ4"),
            Compression::Unknown(x) => write!(f, "unknown compression type (value x (0x{x:x}"),
        }
    }
}

impl std::convert::From<u32> for Compression {
    fn from(x: u32) -> Compression {
        match x {
            0 => Compression::None,
            1 => Compression::LZSS, // probably
            4 => Compression::LZ4,  // probably
            _ => Compression::Unknown(x),
        }
    }
}

impl std::fmt::Display for PkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PkgError::IOError(e) => write!(f, "IO error: {e}"),
            PkgError::Utf8Error(e) => write!(f, "UTF-8 decode error: {e}"),
            PkgError::FilelistError(s) => write!(f, "Error reading file list: {s}"),
            PkgError::DecompressionError(e) => write!(f, "decompression error: {e}"),
            PkgError::CompressionError(e) => write!(f, "decompression error: {e}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileListEntry {
    pub filename: SmolStr,
    pub decompressed_size: u32,
    pub compressed_size: u32,
    pub data_offset: u32,
    pub compression: Compression,
}

pub fn parse_cutf8_padded<F: std::io::Read>(f: &mut F, chunk_size: usize) -> Result<SmolStr, PkgError> {
    let mut data: SmallVec<[u8; 64]> = SmallVec::new();
    let mut i = 0;
    loop {
        let x = read_byte(f).map_err(PkgError::IOError)?;
        i += 1;
        if x == 0 {
            break;
        } else {
            data.push(x);
        }
    }

    // read padding until we hit the chunk size
    while (i % chunk_size) != 0 {
        match read_byte(f) {
            Ok(0) => {}
            Ok(x) => {
                return Err(PkgError::FilelistError(format!("Unexpected padding byte {x}")));
            }
            Err(e) => {
                return Err(PkgError::IOError(e));
            }
        }
        i += 1;
    }

    // check utf-8
    std::str::from_utf8(&data).map(SmolStr::from).map_err(PkgError::Utf8Error)
}

fn parse_filelist_entry<F: std::io::Read>(f: &mut F) -> Result<FileListEntry, PkgError> {
    let filename = parse_cutf8_padded(f, 64)?;
    log::trace!("filename: {filename}");
    let decompressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
    log::trace!("raw size: {decompressed_size} (0x{decompressed_size:x})");
    let compressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
    log::trace!("compressed size: {compressed_size} (0x{compressed_size:x})");
    let data_offset = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
    log::trace!("data offset: {data_offset} (0x{data_offset:x})");
    let compression = Compression::from(u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?));
    log::trace!("compression: {compression:?}");
    Ok(FileListEntry {
        filename,
        decompressed_size,
        compressed_size,
        data_offset,
        compression,
    })
}

fn get_data_lzss<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    file.seek(std::io::SeekFrom::Start(entry.data_offset.into())).map_err(PkgError::IOError)?;

    const LZSS_HEADER_LEN: usize = 12;

    if <u32 as TryInto<usize>>::try_into(entry.decompressed_size).expect("32 bits please") < LZSS_HEADER_LEN {
        return Err(PkgError::DecompressionError(format!(
            "lzss error: compressed size ({}) is too small for lzss header",
            entry.decompressed_size
        )));
    }

    let decompressed_size = u32::from_le_bytes(read_bytes::<4, F>(file).map_err(PkgError::IOError)?);
    let compressed_size = u32::from_le_bytes(read_bytes::<4, F>(file).map_err(PkgError::IOError)?);
    let key = u32::from_le_bytes(read_bytes::<4, F>(file).map_err(PkgError::IOError)?)
        .try_into()
        .map_err(|_| PkgError::DecompressionError(String::from("lzss error: invalid compression key > 255")))?;

    println!("{decompressed_size} {compressed_size}");
    if decompressed_size != entry.decompressed_size {
        panic!();
    }
    if compressed_size != entry.compressed_size {
        panic!();
    }

    let decompressed_size: usize = decompressed_size.try_into().expect("32 bits");
    let compressed_size: usize = compressed_size.try_into().expect("32 bits");

    // TODO: check that sizes make sense? i.e. not too large

    let mut data = std::vec::Vec::<u8>::with_capacity(decompressed_size);

    let mut read = LZSS_HEADER_LEN;
    while read < compressed_size && data.len() <= decompressed_size {
        let byte = read_byte(file).map_err(PkgError::IOError)?;
        read += 1;

        if byte == key {
            log::trace!("chunk");
            // TODO: check that we don't read past compressed_size -> FileSlice struct to ensure this
            let off = read_byte(file).map_err(PkgError::IOError)?;
            read += 1;
            if off == key {
                data.push(key);
            } else {
                let off: usize = if off > key { off - 1 } else { off }.into();
                let len = read_byte(file).map_err(PkgError::IOError)?;
                read += 1;
                for _ in 0..len {
                    let i = data
                        .len()
                        .checked_sub(off)
                        .ok_or_else(|| PkgError::DecompressionError(String::from("lzss error: decompression offset must be <= length of decompressed data")))?;
                    data.push(data[i]);
                }
            }
        } else {
            data.push(byte);
        }
    }

    assert_eq!(read, compressed_size);
    assert_eq!(data.len(), decompressed_size);
    if data.len() != decompressed_size {
        return Err(PkgError::DecompressionError(format!(
            "lzss error: decompressed size ({}) does not match expected value ({})",
            data.len(),
            decompressed_size
        )));
    }

    Ok(data)
}

fn get_data_raw<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    file.seek(std::io::SeekFrom::Start(entry.data_offset.into())).map_err(PkgError::IOError)?;
    if entry.compressed_size != entry.decompressed_size {
        return Err(PkgError::DecompressionError(format!(
            "uncompressed data error: uncompressed size ({}) does not match compressed size ({})",
            entry.decompressed_size, entry.compressed_size
        )));
    }
    let mut data = vec![0; entry.decompressed_size.try_into().expect("32 bits please")];
    file.read_exact(&mut data).map_err(PkgError::IOError)?;
    Ok(data)
}

fn get_data_lz4<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    file.seek(std::io::SeekFrom::Start(entry.data_offset.into())).map_err(PkgError::IOError)?;
    let mut compressed_data = vec![0; entry.compressed_size.try_into().expect("32 bits please")];
    file.read_exact(&mut compressed_data).map_err(PkgError::IOError)?;
    let mut decoder = lz4::Decoder::new(std::io::Cursor::new(&compressed_data)).map_err(PkgError::IOError)?;
    let mut data = vec![0; entry.decompressed_size.try_into().expect("32 bits please")];
    let written = std::io::copy(&mut decoder, &mut std::io::Cursor::new(&mut data)).map_err(PkgError::IOError)?;
    if written != entry.decompressed_size.into() {
        return Err(PkgError::DecompressionError(format!(
            "lz4 error: decompressed size ({}) does not match expected size ({})",
            written, entry.decompressed_size
        )));
    }
    Ok(data)
}

pub fn get_data<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    match entry.compression {
        Compression::None => get_data_raw(file, entry),
        Compression::LZSS => get_data_lzss(file, entry),
        Compression::LZ4 => get_data_lz4(file, entry),
        Compression::Unknown(x) => Err(PkgError::DecompressionError(format!("unsupported/unknown compression type {x}"))),
    }
}
