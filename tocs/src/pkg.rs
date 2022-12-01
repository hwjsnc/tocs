/*
For a description of the pkg file structure, see https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/blob/master/docs/PKG-File-Structure.md
*/

const MAX_DATA_SIZE: u32 = 1 << 30; // a single asset > 1GB is pretty unlikely

use fallible_iterator::FallibleIterator;

pub use crate::compression::Compression;
use crate::io::{read_bytes, PaddedByteString};

pub type FileNameUnchecked = PaddedByteString<64>;
pub type FileName = PaddedAsciiName<64>;

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
    Ok(filelist)
}

#[derive(Debug)]
pub enum PkgError {
    IO(std::io::Error),
    Utf8(std::str::Utf8Error),
    Filename(AsciiNameError),
    Filelist(String),
    Decompression(String),
    Compression(String),
}

impl std::fmt::Display for PkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PkgError::IO(e) => write!(f, "IO error: {e}"),
            PkgError::Utf8(e) => write!(f, "UTF-8 decode error: {e}"),
            PkgError::Filename(e) => write!(f, "pkg file name error: {e}"),
            PkgError::Filelist(s) => write!(f, "pkg file list error: {s}"),
            PkgError::Decompression(e) => write!(f, "decompression error: {e}"),
            PkgError::Compression(e) => write!(f, "decompression error: {e}"),
        }
    }
}

impl std::error::Error for PkgError {}

impl From<crate::io::PaddedStringReadError> for PkgError {
    fn from(error: crate::io::PaddedStringReadError) -> Self {
        match error {
            crate::io::PaddedStringReadError::Io(e) => PkgError::IO(e),
            _ => PkgError::Filelist(error.to_string()),
        }
    }
}

/// entry in the "file list" of the pkg
/// This is the type returned by the parser.
/// The file name is "unchecked" (not really, but not many checks are done) so that we can handle unexpected file names in the original data
#[derive(Clone, Debug)]
pub struct FileListEntry {
    pub filename: FileNameUnchecked,
    pub decompressed_size: u32,
    pub compressed_size: u32,
    pub data_offset: u32,
    pub compression: Compression,
}

fn parse_filelist_entry<F: std::io::Read>(f: &mut F) -> Result<FileListEntry, PkgError> {
    let filename = FileNameUnchecked::read(f).map_err(PkgError::from)?;
    log::trace!("filename: {}", String::from_utf8_lossy(filename.as_bytes_excluding_terminator_and_padding()));
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

fn decompress_data(entry: &FileListEntry, compressed_data: &[u8]) -> Result<std::vec::Vec<u8>, PkgError> {
    entry
        .compression
        .decompress_with_exact_output_size(compressed_data, entry.decompressed_size.try_into().expect("32 bits please"))
        .map_err(PkgError::Decompression)
}

pub fn get_data_raw_from_file<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    check_entry_size(entry)?;
    file.seek(std::io::SeekFrom::Start(entry.data_offset.into())).map_err(PkgError::IO)?;
    let mut compressed_data = vec![0; entry.compressed_size.try_into().expect("32 bits please")];
    file.read_exact(&mut compressed_data).map_err(PkgError::IO)?;
    Ok(compressed_data)
}

pub fn get_data_from_file<F: std::io::Read + std::io::Seek>(file: &mut F, entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    get_data_raw_from_file(file, entry).and_then(|compressed| decompress_data(entry, &compressed))
}

pub fn get_data_raw_from_buffer<'a>(data: &'a [u8], entry: &FileListEntry) -> Result<&'a [u8], PkgError> {
    check_entry_size(entry)?;
    let off1: usize = entry.data_offset.try_into().expect("32 bits please");
    let off2: usize = off1 + <u32 as TryInto<usize>>::try_into(entry.compressed_size).expect("32 bits please");
    let result = &data[off1..off2];
    if result.len().try_into() != Ok(entry.compressed_size) {
        Err(PkgError::Filelist(format!(
            "pkg file '{}' extends past pkg boundary (from {} to {})",
            String::from_utf8_lossy(entry.filename.as_bytes_excluding_terminator_and_padding()),
            off1,
            off2
        )))
    } else {
        Ok(result)
    }
}

pub fn get_data_from_buffer(data: &[u8], entry: &FileListEntry) -> Result<std::vec::Vec<u8>, PkgError> {
    get_data_raw_from_buffer(data, entry).and_then(|compressed| decompress_data(entry, compressed))
}

pub struct Pkg {
    data: std::vec::Vec<u8>,
    filelist: std::vec::Vec<FileListEntry>,
}

impl Pkg {
    pub fn new(data: std::vec::Vec<u8>) -> Result<Self, PkgError> {
        let filelist = load_filelist(&mut std::io::Cursor::new(&data))?;
        Ok(Self { data, filelist })
    }

    pub fn filelist(&self) -> &[FileListEntry] {
        &self.filelist
    }

    pub fn entries(&self) -> impl FallibleIterator<Item = PkgRawEntry<'_>, Error = PkgError> {
        fallible_iterator::convert(self.filelist.iter().map(|entry| {
            Ok(PkgRawEntry {
                metadata: entry,
                data: get_data_raw_from_buffer(&self.data, entry)?,
            })
        }))
    }

    pub fn decompressed_entries(&self) -> impl FallibleIterator<Item = PkgEntry, Error = PkgError> + '_ {
        self.entries().map(|raw_entry| {
            Ok(PkgEntry {
                name: raw_entry.name()?,
                data: raw_entry.decompressed_data()?,
            })
        })
    }
}

pub struct PkgRawEntry<'a> {
    metadata: &'a FileListEntry,
    data: &'a [u8],
}

impl<'a> PkgRawEntry<'a> {
    pub fn name(&self) -> Result<FileName, PkgError> {
        PaddedAsciiName::try_from(&self.metadata.filename).map_err(PkgError::Filename)
    }

    pub fn unchecked_name(&self) -> &FileNameUnchecked {
        &self.metadata.filename
    }

    pub fn compression(&self) -> Compression {
        self.metadata.compression
    }

    pub fn compressed_size(&self) -> u32 {
        self.metadata.compressed_size
    }

    pub fn decompressed_size(&self) -> u32 {
        self.metadata.decompressed_size
    }

    pub fn compressed_data(&self) -> &'a [u8] {
        self.data
    }

    pub fn decompressed_data(&self) -> Result<std::vec::Vec<u8>, PkgError> {
        decompress_data(self.metadata, self.data)
    }
}

pub struct PkgEntry {
    pub name: FileName,
    pub data: std::vec::Vec<u8>,
}

pub fn generate_pkg<S: AsRef<str>, X: AsRef<[u8]>, I: IntoIterator<Item = (S, X)>>(files: I, compression: Compression) -> Result<std::vec::Vec<u8>, PkgError> {
    let mut compressed_files = vec![];
    // TODO: Parallelize this?
    for (n, raw_data) in files {
        let name = FileName::new(n.as_ref()).map_err(PkgError::Filename)?;
        if raw_data.as_ref().len() > MAX_DATA_SIZE.try_into().expect("32 bits please") {
            return Err(PkgError::Compression(format!(
                "pkg file '{}': data size {} exceeds maximum ({})",
                name,
                raw_data.as_ref().len(),
                MAX_DATA_SIZE
            )));
        }
        let compressed = compression
            .compress(raw_data.as_ref())
            .map_err(|e| PkgError::Compression(format!("couldn't compress file '{}': {}", name, e)))?;
        if compressed.len() > MAX_DATA_SIZE.try_into().expect("32 bits please") {
            return Err(PkgError::Compression(format!(
                "pkg file '{}': compressed data size {} exceeds maximum ({})",
                name,
                compressed.len(),
                MAX_DATA_SIZE
            )));
        }
        let decompressed_size: u32 = raw_data.as_ref().len().try_into().expect("32 bits please");
        compressed_files.push(CompressedFile {
            name,
            data: compressed,
            decompressed_size,
            compression,
        });
    }

    generate_pkg_from_compressed_files(&compressed_files)
}

/// name of a file in a .pkg, with a 0 byte terminator and padded to a multiple of the given length
#[derive(Clone, Debug)]
pub struct PaddedAsciiName<const PADDING: usize> {
    name: crate::io::PaddedAsciiString<PADDING>,
}

#[derive(Clone, Debug)]
pub enum AsciiNameError {
    ContainsCtrlChar,
    ContainsWhitespace,
    ContainsPathSep,
    IsEmpty,
    Ascii,
    Utf8(Option<std::str::Utf8Error>),
}

impl From<crate::io::PaddedStringError> for AsciiNameError {
    fn from(error: crate::io::PaddedStringError) -> Self {
        match error {
            crate::io::PaddedStringError::ContainsNull => Self::ContainsCtrlChar,
            crate::io::PaddedStringError::Ascii => Self::Ascii,
            crate::io::PaddedStringError::Utf8(e) => Self::Utf8(Some(e)),
        }
    }
}

impl std::fmt::Display for AsciiNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::ContainsCtrlChar => write!(f, "name contains control character(s)"),
            Self::ContainsWhitespace => write!(f, "name contains whitespace (other than literal space)"),
            Self::ContainsPathSep => write!(f, "name contains a path separator"),
            Self::IsEmpty => write!(f, "name is empty"),
            Self::Ascii => write!(f, "name is not valid ASCII"),
            Self::Utf8(Some(e)) => write!(f, "UTF-8 error: {e}"),
            Self::Utf8(None) => write!(f, "UTF-8 conversion error"),
        }
    }
}

impl std::error::Error for AsciiNameError {}

impl<const PADDING: usize> PaddedAsciiName<PADDING> {
    pub fn new(name: &str) -> Result<Self, AsciiNameError> {
        let s = crate::io::PaddedAsciiString::<PADDING>::new(name).map_err(AsciiNameError::from)?;
        if name.chars().any(|c| c.is_ascii_control()) {
            Err(AsciiNameError::ContainsCtrlChar)
        } else if name.chars().any(|c| c.is_ascii_whitespace() && c != ' ') {
            Err(AsciiNameError::ContainsWhitespace)
        } else if name.contains('/') || name.contains('\\') {
            Err(AsciiNameError::ContainsPathSep)
        } else if name.is_empty() {
            Err(AsciiNameError::IsEmpty)
        } else {
            if s.len_with_padding() > PADDING {
                log::warn!(
                    "file name length ({}) exceeds padding block size ({}), idk if that'll work but you can try shrug emoji",
                    s.len_with_padding(),
                    PADDING
                );
            }
            Ok(Self { name: s })
        }
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Result<Self, AsciiNameError> {
        std::str::from_utf8(bytes).map_err(|e| AsciiNameError::Utf8(Some(e))).and_then(|str| Self::new(str))
    }

    pub fn try_from_os_str(name: &std::ffi::OsStr) -> Result<Self, AsciiNameError> {
        match name.to_str() {
            Some(s) => Self::new(s),
            None => Err(AsciiNameError::Utf8(None)),
        }
    }

    pub fn try_from_path(path: &std::path::Path) -> Result<Self, AsciiNameError> {
        Self::try_from_os_str(path.as_os_str())
    }

    pub fn len_with_padding(&self) -> usize {
        self.name.len_with_padding()
    }

    pub fn write_to<F: std::io::Write>(&self, f: &mut F) -> std::io::Result<usize> {
        self.name.write_to(f)
    }

    pub fn as_bytes_excluding_terminator_and_padding(&self) -> &[u8] {
        self.name.as_bytes_excluding_terminator_and_padding()
    }

    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }

    pub fn as_path(&self) -> &std::path::Path {
        let p = &std::path::Path::new(self.as_str());
        assert_eq!(p.components().count(), 1); // no path separators
        p
    }
}

impl<const PADDING: usize> TryFrom<&crate::io::PaddedByteString<PADDING>> for PaddedAsciiName<PADDING> {
    type Error = AsciiNameError;
    fn try_from(s: &crate::io::PaddedByteString<PADDING>) -> Result<Self, Self::Error> {
        Self::new_from_bytes(s.as_bytes_excluding_terminator_and_padding())
    }
}

impl<const PADDING: usize> TryFrom<crate::io::PaddedByteString<PADDING>> for PaddedAsciiName<PADDING> {
    type Error = AsciiNameError;
    fn try_from(s: crate::io::PaddedByteString<PADDING>) -> Result<Self, Self::Error> {
        Self::try_from(&s)
    }
}

impl<const PADDING: usize> std::fmt::Display for PaddedAsciiName<PADDING> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

pub struct CompressedFile {
    pub name: FileName,
    pub data: std::vec::Vec<u8>,
    pub decompressed_size: u32,
    pub compression: Compression,
}

pub fn generate_pkg_from_compressed_files(files: &[CompressedFile]) -> Result<std::vec::Vec<u8>, PkgError> {
    // precompute length of header/metadata because we'll need it later to insert the data offsets while constructing the header/metadata
    let mut length_of_header = 8;
    for f in files.iter() {
        if f.data.len() > MAX_DATA_SIZE.try_into().expect("32 bits please") {
            return Err(PkgError::Filelist(format!("File '{}' compressed data size exceeds max ({})", f.name.as_str(), MAX_DATA_SIZE)));
        }
        length_of_header += f.name.len_with_padding() + 4 * 4;
    }

    let data_offsets = {
        let mut data_offsets: std::vec::Vec<u32> = vec![];
        let mut off: u32 = length_of_header.try_into().map_err(|_| PkgError::Filelist(String::from("header is too long")))?;
        for f in files.iter() {
            data_offsets.push(off);
            off = <usize as TryInto<u32>>::try_into(f.data.len())
                .ok()
                .and_then(|len| len.checked_add(off))
                .ok_or_else(|| PkgError::Filelist(format!("pkg file '{}': data offset too big", f.name)))?;
        }
        data_offsets
    };

    // write header
    let mut output: std::vec::Vec<u8> = vec![];
    output.extend_from_slice(&[0, 0, 0, 0]); // checksum or something, ignored?
    let number_of_files: u32 = files.len().try_into().map_err(|_| PkgError::Filelist(String::from("too many files")))?;
    output.extend_from_slice(&number_of_files.to_le_bytes());

    // write metadata for each file
    for (f, offset) in files.iter().zip(data_offsets.iter()) {
        f.name.write_to(&mut output).expect("writing to output vector should succeed");
        output.extend_from_slice(&f.decompressed_size.to_le_bytes());
        let compressed_size: u32 = f.data.len().try_into().map_err(|_| PkgError::Filelist(format!("compressed data size ({}) too big", f.data.len())))?;
        output.extend_from_slice(&compressed_size.to_le_bytes());
        output.extend_from_slice(&offset.to_le_bytes());
        let compression: u32 = f.compression.into();
        output.extend_from_slice(&compression.to_le_bytes());
    }

    assert_eq!(output.len(), length_of_header);

    // write data
    for (f, offset) in files.iter().zip(data_offsets.iter()) {
        assert_eq!(output.len().try_into(), Ok(*offset));
        output.extend_from_slice(&f.data);
    }

    Ok(output)
}
