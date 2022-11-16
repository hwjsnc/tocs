/*
For a description of the pka file format, see https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/pull/4/files

note:
- files are mostly (but not fully) sorted alphabetically
- hashes are sorted lexicographically
- file offset sorting is unknown
*/

const MAX_DATA_SIZE: u32 = 1 << 30; // a single asset > 1GB is pretty unlikely

use smol_str::SmolStr;

use crate::compression::Compression;

use crate::io::read_bytes;
pub use crate::pkg::PkgError;

#[derive(Clone, Debug)]
pub struct PkgListEntry {
    pub name: SmolStr,
    pub filenames: std::vec::Vec<SmolStr>,
    pub hashes: std::vec::Vec<[u8; 32]>,
}

#[derive(Clone, Debug)]
pub struct FileDataEntry {
    pub hash: [u8; 32],
    pub offset: u64,
    pub compressed_size: u32,
    pub decompressed_size: u32,
    pub compression: Compression,
}

fn parse_filename<F: std::io::Read>(f: &mut F, padding: usize) -> Result<SmolStr, PkgError> {
    crate::io::read_cutf8_padded(f, padding).map_err(|e| match e {
        crate::io::ReadUtf8Error::IOError(e) => PkgError::IOError(e),
        crate::io::ReadUtf8Error::Utf8Error(e) => PkgError::Utf8Error(e),
        crate::io::ReadUtf8Error::UnexpectedPaddingError(x) => PkgError::FilelistError(format!("unexpected padding byte {x} (0x{x:x})")),
    })
}

pub fn load_filelist<F: std::io::Read>(f: &mut F) -> Result<(std::vec::Vec<PkgListEntry>, std::vec::Vec<FileDataEntry>), PkgError> {
    // header
    let data = read_bytes::<4, _>(f).map_err(PkgError::IOError)?; // TODO: do we expect a certain value?
    let number_of_pkgs = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
    log::debug!("number of pkgs in pka: {number_of_pkgs}");

    // list of pkg filenames and hashes
    let mut pkgs = std::vec::Vec::with_capacity(number_of_pkgs.try_into().expect("32 bits please"));
    for i in 0..number_of_pkgs {
        log::trace!("reading pkg {} of {}", i + 1, number_of_pkgs);
        let mut filenames = vec![];
        let mut hashes = vec![];
        let pkg_filename = parse_filename(f, 32)?;
        log::trace!("pkg filename: {pkg_filename}");
        let number_of_files = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
        log::trace!("number of files in pkg: {number_of_files}");
        for j in 0..number_of_files {
            log::trace!("reading file {} of {}", j + 1, number_of_files);
            let filename = parse_filename(f, 64)?;
            log::trace!("file name: {filename}");
            let hash = read_bytes::<32, _>(f).map_err(PkgError::IOError)?;
            log::trace!("file hash: {}", hex::encode(hash));
            filenames.push(filename);
            hashes.push(hash);
        }
        pkgs.push(PkgListEntry {
            name: pkg_filename,
            filenames,
            hashes,
        });
    }

    let number_of_hashes = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
    log::debug!("number of hashes: {}", number_of_hashes);

    // TODO: use a hashmap or indexmap instead?
    // content hashes, locations, sizes, compression
    let mut filedata = std::vec::Vec::with_capacity(number_of_hashes.try_into().expect("32 bits please"));
    for i in 0..number_of_hashes {
        let hash = read_bytes::<32, _>(f).map_err(PkgError::IOError)?;
        let offset = u64::from_le_bytes(read_bytes::<8, _>(f).map_err(PkgError::IOError)?);
        let compressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
        let decompressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?);
        let compression = Compression::from(u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IOError)?));
        log::trace!(
            "hash {} of {}: hash = {}, offset = {} (0x{:x}), compressed size = {} (0x{:x}), decompressed size = {}, compression = {}",
            i + 1,
            number_of_hashes,
            hex::encode(hash),
            offset,
            offset,
            compressed_size,
            compressed_size,
            decompressed_size,
            compression
        );
        filedata.push(FileDataEntry {
            hash,
            offset,
            compressed_size,
            decompressed_size,
            compression,
        });
    }

    Ok((pkgs, filedata))
}

pub fn get_blob_data_raw<F: std::io::Read + std::io::Seek>(file: &mut F, blob: &FileDataEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    if blob.compressed_size > MAX_DATA_SIZE {
        let x = blob.compressed_size;
        return Err(PkgError::DecompressionError(format!(
            "error reading compressed data: size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )));
    }
    file.seek(std::io::SeekFrom::Start(blob.offset.into())).map_err(PkgError::IOError)?;
    let mut compressed_data = vec![0; blob.compressed_size.try_into().expect("32 bits please")];
    file.read_exact(&mut compressed_data).map_err(PkgError::IOError)?;
    Ok(compressed_data)
}

pub fn get_blob_data<F: std::io::Read + std::io::Seek>(file: &mut F, blob: &FileDataEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    if blob.compressed_size > MAX_DATA_SIZE {
        let x = blob.compressed_size;
        return Err(PkgError::DecompressionError(format!(
            "decompression error: compressed size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )));
    }
    if blob.decompressed_size > MAX_DATA_SIZE {
        let x = blob.decompressed_size;
        return Err(PkgError::DecompressionError(format!(
            "decompression error: decompressed size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )));
    }

    let compressed_data = get_blob_data_raw(file, blob)?;

    blob.compression
        .decompress_with_exact_output_size(&compressed_data, blob.decompressed_size.try_into().expect("32 bits please"))
        .map_err(PkgError::DecompressionError)
}

// TODO: get data by hash, get data by file name
