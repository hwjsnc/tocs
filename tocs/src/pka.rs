/*
For a description of the pka file format, see https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/pull/4/files

note:
- files are mostly (but not fully) sorted alphabetically
- hashes are sorted lexicographically
- file offset sorting is unknown
*/

const MAX_DATA_SIZE: u32 = 1 << 30; // a single asset > 1GB is pretty unlikely

use fallible_iterator::FallibleIterator;

use sha2::Digest as _;

use std::borrow::Borrow;
use std::io::Seek as _;
use std::io::Write as _;

use crate::compression::Compression;
use crate::io::{read_bytes, PaddedByteString};
use crate::pkg::PaddedAsciiName;

pub use crate::pkg::{FileName, FileNameUnchecked, PkgError};

pub type PkgNameUnchecked = PaddedByteString<32>;
pub type PkgName = PaddedAsciiName<32>;

#[derive(Clone, Debug)]
pub struct PkgListEntryUnchecked {
    pub name: PkgNameUnchecked,
    pub files: std::vec::Vec<PkgFileListEntryUnchecked>,
}

#[derive(Clone, Debug)]
pub struct PkgFileListEntryUnchecked {
    pub name: FileNameUnchecked,
    pub hash: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct PkgListEntry {
    pub name: PkgName,
    pub files: std::vec::Vec<PkgFileListEntry>,
}

#[derive(Clone, Debug)]
pub struct PkgFileListEntry {
    pub name: FileName,
    pub hash: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct FileDataEntry {
    pub hash: [u8; 32],
    pub offset: u64,
    pub compressed_size: u32,
    pub decompressed_size: u32,
    pub compression: Compression,
}

pub fn load_filelist<F: std::io::Read>(f: &mut F) -> Result<(std::vec::Vec<PkgListEntryUnchecked>, std::vec::Vec<FileDataEntry>), PkgError> {
    // header
    let magic = read_bytes::<4, _>(f).map_err(PkgError::IO)?;
    if magic != [0x0d, 0xcf, 0xf7, 0x7f] {
        return Err(PkgError::Filelist(format!("unexpected magic bytes: {magic:?}")));
    }
    let number_of_pkgs = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
    log::debug!("number of pkgs in pka: {number_of_pkgs}");

    // list of pkg filenames and hashes
    let mut pkgs = std::vec::Vec::with_capacity(number_of_pkgs.try_into().expect("32 bits please"));
    for i in 0..number_of_pkgs {
        log::trace!("reading pkg {} of {}", i + 1, number_of_pkgs);
        let mut files = vec![];
        let pkg_filename = PkgNameUnchecked::read(f).map_err(|e| PkgError::Filelist(e.to_string()))?;
        log::trace!("pkg filename: {}", pkg_filename.to_utf8_lossy());
        let number_of_files = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
        log::trace!("number of files in pkg: {number_of_files}");
        for j in 0..number_of_files {
            log::trace!("reading file {} of {}", j + 1, number_of_files);
            let filename = FileNameUnchecked::read(f)?;
            log::trace!("file name: {}", filename.to_utf8_lossy());
            let hash = read_bytes::<32, _>(f).map_err(PkgError::IO)?;
            log::trace!("file hash: {}", hex::encode(hash));
            files.push(PkgFileListEntryUnchecked { name: filename, hash });
        }
        pkgs.push(PkgListEntryUnchecked { name: pkg_filename, files });
    }

    let number_of_hashes = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
    log::debug!("number of hashes: {}", number_of_hashes);

    // TODO: use a hashmap or indexmap instead?
    // content hashes, locations, sizes, compression
    let mut filedata = std::vec::Vec::with_capacity(number_of_hashes.try_into().expect("32 bits please"));
    for i in 0..number_of_hashes {
        let hash = read_bytes::<32, _>(f).map_err(PkgError::IO)?;
        let offset = u64::from_le_bytes(read_bytes::<8, _>(f).map_err(PkgError::IO)?);
        let compressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
        let decompressed_size = u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?);
        let compression = Compression::from(u32::from_le_bytes(read_bytes::<4, _>(f).map_err(PkgError::IO)?));
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
        return Err(PkgError::Decompression(format!("error reading compressed data: size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})")));
    }
    file.seek(std::io::SeekFrom::Start(blob.offset)).map_err(PkgError::IO)?;
    let mut compressed_data = vec![0; blob.compressed_size.try_into().expect("32 bits please")];
    file.read_exact(&mut compressed_data).map_err(PkgError::IO)?;
    Ok(compressed_data)
}

pub fn get_blob_data<F: std::io::Read + std::io::Seek>(file: &mut F, blob: &FileDataEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    if blob.compressed_size > MAX_DATA_SIZE {
        let x = blob.compressed_size;
        return Err(PkgError::Decompression(format!(
            "decompression error: compressed size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )));
    }
    if blob.decompressed_size > MAX_DATA_SIZE {
        let x = blob.decompressed_size;
        return Err(PkgError::Decompression(format!(
            "decompression error: decompressed size {x} (0x{x:x}) exceeds max size ({MAX_DATA_SIZE})"
        )));
    }

    let compressed_data = get_blob_data_raw(file, blob)?;

    blob.compression
        .decompress_with_exact_output_size(&compressed_data, blob.decompressed_size.try_into().expect("32 bits please"))
        .map_err(PkgError::Decompression)
}

// TODO: get data by hash, get data by file name

pub fn hash_data(data: &[u8]) -> [u8; 32] {
    let sha256 = sha2::Sha256::digest(data);
    sha256.as_slice().try_into().expect("SHA256 result should be 32 byte long")
}

pub fn hash_compressed_data(data: &[u8], compression: Compression, decompressed_size: u32) -> Result<[u8; 32], String> {
    // we could try to stream the decompressed data into the decompression algorithm, but the files should be small enough for this to not be necessary
    let decompressed_data = compression.decompress_with_exact_output_size(data, decompressed_size.try_into().expect("32 bits please"))?;
    Ok(hash_data(&decompressed_data))
}

#[derive(Debug)]
pub enum PkaWriteError<E> {
    IO(std::io::Error),
    Metadata(String),
    Missing([u8; 32]),
    Blob(E),
    Compression(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CompressedBlobMetadata {
    pub hash: [u8; 32],
    pub decompressed_size: u32,
    pub compressed_size: u32,
    pub compression: Compression,
}

impl<E: std::fmt::Display> std::fmt::Display for PkaWriteError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PkaWriteError::IO(e) => write!(f, "IO error: {e}"),
            PkaWriteError::Metadata(s) => write!(f, "metadata error: {s}"),
            PkaWriteError::Missing(hash) => write!(f, "hash {} is present in metadata, but there is no such blob", hex::encode(hash)),
            PkaWriteError::Blob(e) => write!(f, "blob read error: {e}"),
            PkaWriteError::Compression(e) => write!(f, "decompression error: {e}"),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for PkaWriteError<E> {}

pub fn write_pka_from_compressed_data<E, X: Borrow<[u8]>, I: FallibleIterator<Item = ([u8; 32], X), Error = E>>(
    path: &std::path::Path, // path to output file
    overwrite_if_exists: bool,
    pkgs: std::vec::Vec<PkgListEntry>,   // TODO: take a struct with PaddedFileNames?
    blob_order: std::vec::Vec<[u8; 32]>, // order of actual data in the output file; missing or superfluous hashes are fine
    blobinfo: std::vec::Vec<CompressedBlobMetadata>,
    mut blobdata: I, // this should be a streaming iterator to save memory (all blobs together are >10 GB)
) -> Result<(), PkaWriteError<E>> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(overwrite_if_exists)
        .create_new(!overwrite_if_exists)
        .open(path)
        .map_err(PkaWriteError::IO)?;

    // generate pkg list data (small enough to fit in main memory)
    let pkg_list_data = {
        log::debug!("generating pkg list data");
        let mut data = vec![];
        data.extend_from_slice(&[0x0d, 0xcf, 0xf7, 0x7f]); // magic bytes

        let number_of_pkgs: u32 = pkgs.len().try_into().expect("32 bits please");
        data.extend_from_slice(&number_of_pkgs.to_le_bytes());

        // we could sort the pkgs and filenames here, but it doesn't seem necessary
        for pkg in pkgs {
            pkg.name.write_to(&mut data).expect("writing to vector shouldn't fail");
            let number_of_files: u32 = pkg.files.len().try_into().map_err(|_| PkaWriteError::Metadata("too many files in pkg".into()))?;
            data.extend_from_slice(&number_of_files.to_le_bytes());
            for PkgFileListEntry { name, hash } in pkg.files.iter() {
                name.write_to(&mut data).expect("writing to vector shouldn't fail");
                data.extend_from_slice(hash);
            }
        }

        data
    };

    // compute offsets of each blob (and total file size)
    let (file_data_list, total_file_size) = {
        log::debug!("computing blob offsets");
        // order blobs by how they appear in the blob_order.txt file
        // blobs that are missing from there will be sorted to the end
        let blob_index: std::collections::HashMap<[u8; 32], usize> = blob_order.into_iter().enumerate().map(|(i, hash)| (hash, i)).collect();
        let mut blobinfo = blobinfo;
        blobinfo.sort_unstable_by(|a, b| match (blob_index.get(&a.hash), blob_index.get(&b.hash)) {
            (Some(i), Some(j)) => i.cmp(j),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.hash.cmp(&b.hash),
        });

        let mut off: u64 = 4; // for the number of hashes
        off += <usize as TryInto<u64>>::try_into(pkg_list_data.len()).expect("less than 64 bits please");
        off += 52 * <usize as TryInto<u64>>::try_into(blobinfo.len()).expect("less than 64 bits please"); // blob info list takes 52 bytes per entry
        let mut file_data_list = std::collections::BTreeMap::<[u8; 32], FileDataEntry>::new(); // This is a BTreeMap so that we iterate the values in order of ascending hash values
        for blob in blobinfo.iter() {
            let entry = FileDataEntry {
                hash: blob.hash,
                offset: off,
                decompressed_size: blob.decompressed_size,
                compressed_size: blob.compressed_size,
                compression: blob.compression,
            };
            if file_data_list.insert(blob.hash, entry).is_some() {
                return Err(PkaWriteError::Metadata(format!("Duplicate metadata for blob with hash {}", hex::encode(blob.hash))));
            }
            off = off
                .checked_add(blob.compressed_size.into())
                .ok_or_else(|| PkaWriteError::Metadata("total size of blobs is too big".to_string()))?;
        }

        (file_data_list, off)
    };

    // generate blob list data (small enough to fit in main memory)
    let blob_list_data = {
        log::debug!("generating file data list");
        let number_of_hashes: u32 = file_data_list.len().try_into().expect("32 bits please");
        let mut data = std::vec::Vec::with_capacity(52 * file_data_list.len());
        data.extend_from_slice(&number_of_hashes.to_le_bytes());
        for entry in file_data_list.values() {
            // note: file_data_list is a tree map so we iterate in key order, i.e. by ascending data hash
            data.extend_from_slice(&entry.hash);
            data.extend_from_slice(&entry.offset.to_le_bytes());
            data.extend_from_slice(&entry.compressed_size.to_le_bytes());
            data.extend_from_slice(&entry.decompressed_size.to_le_bytes());
            data.extend_from_slice(&u32::from(entry.compression).to_le_bytes());
        }
        data
    };

    // write header
    file.write(&pkg_list_data).map_err(PkaWriteError::IO)?;
    file.write(&blob_list_data).map_err(PkaWriteError::IO)?;

    file.set_len(total_file_size).map_err(PkaWriteError::IO)?;

    // write data blobs
    let mut written_hashes = std::collections::HashSet::new();
    while let Some((hash, blob)) = blobdata.next().map_err(PkaWriteError::Blob)? {
        if let Some(FileDataEntry { offset, .. }) = file_data_list.get(&hash) {
            if written_hashes.insert(hash) {
                // we only need to write each blob once
                file.seek(std::io::SeekFrom::Start(*offset)).map_err(PkaWriteError::IO)?;
                file.write(blob.borrow()).map_err(PkaWriteError::IO)?;
                file.seek(std::io::SeekFrom::Start(*offset)).map_err(PkaWriteError::IO)?;
            }
        } else {
            return Err(PkaWriteError::Metadata(format!("File metadata is missing blob with hash '{}'", hex::encode(hash))));
        }
    }

    if let Some(missing) = file_data_list.keys().find(|hash| !written_hashes.contains(*hash)) {
        return Err(PkaWriteError::Missing(*missing));
    }

    Ok(())
}
