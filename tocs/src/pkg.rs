/*
For a description of the pkg file structure, see https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/blob/master/docs/PKG-File-Structure.md
*/

// TODO: actually use this, haha
const MAX_DATA_SIZE: u32 = 1 << 30; // a single asset > 1GB is pretty unlikely

use smol_str::SmolStr;

pub use crate::compression::Compression;
use crate::io::read_bytes;

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

#[derive(Debug)]
pub enum PkgError {
    IOError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    FilelistError(String),
    DecompressionError(String),
    CompressionError(String),
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

impl std::error::Error for PkgError {}

#[derive(Clone, Debug)]
pub struct FileListEntry {
    pub filename: SmolStr,
    pub decompressed_size: u32,
    pub compressed_size: u32,
    pub data_offset: u32,
    pub compression: Compression,
}

fn parse_filename<F: std::io::Read>(f: &mut F) -> Result<SmolStr, PkgError> {
    crate::io::read_cutf8_padded(f, 64).map_err(|e| match e {
        crate::io::ReadUtf8Error::IOError(e) => PkgError::IOError(e),
        crate::io::ReadUtf8Error::Utf8Error(e) => PkgError::Utf8Error(e),
        crate::io::ReadUtf8Error::UnexpectedPaddingError(x) => PkgError::FilelistError(format!("unexpected padding byte {x} (0x{x:x})")),
    })
}

fn parse_filelist_entry<F: std::io::Read>(f: &mut F) -> Result<FileListEntry, PkgError> {
    let filename = parse_filename(f)?;
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

pub fn get_data_raw<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    if entry.compressed_size > MAX_DATA_SIZE {
        let x = entry.compressed_size;
        return Err(PkgError::DecompressionError(format!(
            "error reading compressed data: size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )));
    }
    file.seek(std::io::SeekFrom::Start(entry.data_offset.into())).map_err(PkgError::IOError)?;
    let mut compressed_data = vec![0; entry.compressed_size.try_into().expect("32 bits please")];
    file.read_exact(&mut compressed_data).map_err(PkgError::IOError)?;
    Ok(compressed_data)
}

pub fn get_data<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    if entry.compressed_size > MAX_DATA_SIZE {
        let x = entry.compressed_size;
        return Err(PkgError::DecompressionError(format!(
            "decompression error: compressed size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )));
    }
    if entry.decompressed_size > MAX_DATA_SIZE {
        let x = entry.decompressed_size;
        return Err(PkgError::DecompressionError(format!(
            "decompression error: decompressed size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )));
    }

    let compressed_data = get_data_raw(file, entry)?;

    entry
        .compression
        .decompress_with_exact_output_size(&compressed_data, entry.decompressed_size.try_into().expect("32 bits please"))
        .map_err(PkgError::DecompressionError)
}
