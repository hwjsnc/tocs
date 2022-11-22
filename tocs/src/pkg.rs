/*
For a description of the pkg file structure, see https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/blob/master/docs/PKG-File-Structure.md
*/

const MAX_DATA_SIZE: u32 = 1 << 30; // a single asset > 1GB is pretty unlikely

use smol_str::SmolStr;

pub use crate::compression::Compression;
use crate::io::read_bytes;

pub fn load_filelist<F: std::io::Read>(f: &mut F) -> Result<std::vec::Vec<FileListEntry>, PkgError> {
    let _data = read_bytes::<4, _>(f).map_err(PkgError::IO)?; // TODO: magic header or a hash or something???
    let number_of_files = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
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
    IO(std::io::Error),
    Utf8(std::str::Utf8Error),
    Filelist(String),
    Decompression(String),
    Compression(String),
}

impl std::fmt::Display for PkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PkgError::IO(e) => write!(f, "IO error: {e}"),
            PkgError::Utf8(e) => write!(f, "UTF-8 decode error: {e}"),
            PkgError::Filelist(s) => write!(f, "pkg file list error: {s}"),
            PkgError::Decompression(e) => write!(f, "decompression error: {e}"),
            PkgError::Compression(e) => write!(f, "decompression error: {e}"),
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
        crate::io::ReadUtf8Error::IO(e) => PkgError::IO(e),
        crate::io::ReadUtf8Error::Utf8(e) => PkgError::Utf8(e),
        crate::io::ReadUtf8Error::UnexpectedPadding(x) => PkgError::Filelist(format!("unexpected padding byte {x} (0x{x:x})")),
    })
}

fn parse_filelist_entry<F: std::io::Read>(f: &mut F) -> Result<FileListEntry, PkgError> {
    let filename = parse_filename(f)?;
    log::trace!("filename: {filename}");
    let decompressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
    log::trace!("raw size: {decompressed_size} (0x{decompressed_size:x})");
    let compressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
    log::trace!("compressed size: {compressed_size} (0x{compressed_size:x})");
    let data_offset = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
    log::trace!("data offset: {data_offset} (0x{data_offset:x})");
    let compression = Compression::from(u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?));
    log::trace!("compression: {compression:?}");
    Ok(FileListEntry {
        filename,
        decompressed_size,
        compressed_size,
        data_offset,
        compression,
    })
}

fn check_entry_size(entry: &FileListEntry) -> Result<(), PkgError> {
    if entry.compressed_size > MAX_DATA_SIZE {
        let x = entry.compressed_size;
        Err(PkgError::Decompression(format!("error reading compressed data: size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})")))
    } else if entry.decompressed_size > MAX_DATA_SIZE {
        let x = entry.decompressed_size;
        Err(PkgError::Decompression(format!(
            "decompression error: decompressed size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )))
    } else {
        Ok(())
    }
}

pub fn get_data_raw_from_file<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    check_entry_size(entry)?;
    file.seek(std::io::SeekFrom::Start(entry.data_offset.into())).map_err(PkgError::IO)?;
    let mut compressed_data = vec![0; entry.compressed_size.try_into().expect("32 bits please")];
    file.read_exact(&mut compressed_data).map_err(PkgError::IO)?;
    Ok(compressed_data)
}

pub fn get_data_from_file<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    let compressed_data = get_data_raw_from_file(file, entry)?;
    entry
        .compression
        .decompress_with_exact_output_size(&compressed_data, entry.decompressed_size.try_into().expect("32 bits please"))
        .map_err(PkgError::Decompression)
}

pub fn get_data_raw_from_buffer<'a>(data: &'a [u8], entry: &FileListEntry) -> Result<&'a [u8], PkgError> {
    check_entry_size(entry)?;
    let off1: usize = entry.data_offset.try_into().expect("32 bits please");
    let off2: usize = off1 + <u32 as TryInto<usize>>::try_into(entry.compressed_size).expect("32 bits please");
    Ok(&data[off1..off2])
}

pub fn get_data_from_buffer(data: &[u8], entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    let compressed_data = get_data_raw_from_buffer(data, entry)?;
    entry
        .compression
        .decompress_with_exact_output_size(compressed_data, entry.decompressed_size.try_into().expect("32 bits please"))
        .map_err(PkgError::Decompression)
}

/// Length of the given string in bytes, when serialized as a C string (i.e. with a trailing null byte) and padded up to a length that's a multiple of padding.
pub(crate) fn padded_length(s: &str, padding: usize) -> usize {
    assert!(padding > 1);
    let (p, q) = num_integer::div_rem(s.len() + 1, padding);
    if q == 0 {
        p * padding
    } else {
        (p + 1) * padding
    }
}

/// Extend the name with a zero byte and pad it for use with .pkg files etc; also checks for validity of the file name
pub(crate) fn pad_name(name: &str, padding: usize) -> Result<impl AsRef<[u8]>, PkgError> {
    assert!(padding > 1);
    if !name.is_ascii() {
        Err(PkgError::Filelist(format!("name '{}' is not ASCII", name.escape_default())))
    } else if name.chars().any(|c| c.is_ascii_control()) {
        Err(PkgError::Filelist(format!("name '{}' contains control characters", name.escape_default())))
    } else if name.chars().any(|c| c.is_ascii_whitespace() && c != ' ') {
        Err(PkgError::Filelist(format!("name '{}' contains whitespace (other than literal space)", name.escape_default())))
    } else if name.contains('/') || name.contains('\\') {
        Err(PkgError::Filelist(format!("name '{}' contains path separators", name.escape_default())))
    } else {
        if name.len() + 1 > padding {
            log::warn!("name length exceeds padding block size, idk if that'll work shrug emoji");
        }
        let n_zeros = (name.len() / padding + 1) * padding - name.len();
        let mut result = smallvec::SmallVec::<[u8; 64]>::from(name.as_ref()); // 64 bytes is the largest padding I've seen and it's not too big as a default
        for _ in 0..n_zeros {
            result.push(0);
        }
        assert_eq!(result.len() % padding, 0);
        assert!(result.len() > name.len());
        assert!(result.len() < name.len() + padding);
        Ok(result)
    }
}

pub fn generate_pkg<S: AsRef<str>, X: AsRef<[u8]>>(files: &[(S, X)], compression: Compression) -> Result<std::vec::Vec<u8>, PkgError> {
    const PADDING: usize = 64;

    // precompute length of header/metadata because we'll need it later to insert the data offsets while constructing the header/metadata
    let length_of_header: usize = files.iter().map(|(name, _)| padded_length(name.as_ref(), PADDING) + 4 * 4).sum();
    let length_of_header: usize = length_of_header + 8;

    // compress data and compute data offsets, starting from the end of the header/metadata
    // pkg files are not super large (the biggest one are ~100MB, significantly smaller than 1 GB)
    // It should be ok to store both the full uncompressed and full compressed data at the same time in main memory
    let mut compressed_data: std::vec::Vec<std::vec::Vec<u8>> = vec![];
    let mut data_offsets = vec![];
    for (_, data) in files {
        let x = compression.compress(data.as_ref()).map_err(PkgError::Compression)?;
        let offset = data_offsets
            .last()
            .unwrap_or(&length_of_header)
            .checked_add(compressed_data.last().map(|x| x.len()).unwrap_or(0))
            .ok_or_else(|| PkgError::Filelist("compressed data is too big".to_string()))?;
        compressed_data.push(x);
        data_offsets.push(offset);
    }
    let compressed_data = compressed_data;
    let data_offsets = data_offsets;

    // write header
    let mut output: std::vec::Vec<u8> = vec![];
    output.extend_from_slice(&[0, 0, 0, 0]); // checksum or something, ignored?
    let number_of_files: u32 = files.len().try_into().expect("too many files");
    output.extend_from_slice(&number_of_files.to_le_bytes());

    // write metadata for each file
    for ((name, raw_data), compressed_data, offset) in itertools::izip!(files.iter(), compressed_data.iter(), data_offsets.iter()) {
        let padded_name = pad_name(name.as_ref(), PADDING)?;
        output.extend_from_slice(padded_name.as_ref());
        let decompressed_size: u32 = raw_data
            .as_ref()
            .len()
            .try_into()
            .map_err(|_| PkgError::Filelist(format!("decompressed data size ({}) too big", raw_data.as_ref().len())))?;
        output.extend_from_slice(&decompressed_size.to_le_bytes());
        let compressed_size: u32 = compressed_data
            .len()
            .try_into()
            .map_err(|_| PkgError::Filelist(format!("compressed data size ({}) too big", compressed_data.len())))?;
        output.extend_from_slice(&compressed_size.to_le_bytes());
        let data_offset: u32 = (*offset).try_into().map_err(|_| PkgError::Filelist(format!("data offset is ({}) too big", offset)))?;
        output.extend_from_slice(&data_offset.to_le_bytes());
        let compression: u32 = compression.into();
        output.extend_from_slice(&compression.to_le_bytes());
    }

    assert_eq!(output.len(), length_of_header);

    // write data
    for (x, off) in compressed_data.iter().zip(data_offsets.iter()) {
        assert_eq!(output.len(), *off);
        output.extend_from_slice(x);
    }

    Ok(output)
}
