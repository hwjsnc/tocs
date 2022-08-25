/*
For a description of the pkg file structure, see https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/blob/master/docs/PKG-File-Structure.md
*/

use crate::{parse_bytes, parse_eof, parse_le_u32, parse_utf8cstring_padded, ParseContext};

pub enum Compression {
    None,
    /*
    LZSS,
    LZ4,
    Zstandard,
    */
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub detail: ParseErrorDetail,
    pub context: ParseErrorContext,
}

#[derive(Clone, Debug)]
pub enum ParseErrorDetail {
    Primitive(crate::PrimitiveParseError),
    UnexpectedHeaderValue { value: u32 },
    UnknownCompressionType { value: u32 },
    InvalidFileContentRange { start: u32, length: u32, end: usize },
    DataRangeOverflowError,
}

#[derive(Clone, Debug)]
pub enum ParseErrorContext {
    PkgHeader,
    PkgFileList { entry_number: u32, total_number_of_entries: u32 },
    PkgFileData { entry_number: u32, total_number_of_entries: u32 },
}

impl ParseErrorDetail {
    pub fn to_msg(&self, input: &[u8]) -> String {
        match self {
            ParseErrorDetail::Primitive(p) => p.to_msg(input),
            ParseErrorDetail::UnexpectedHeaderValue { value } => format!("unexpected value {} at start of pkg file header", value),
            ParseErrorDetail::UnknownCompressionType { value } => format!("unexpected compression value {}", value),
            ParseErrorDetail::InvalidFileContentRange { start, length, end } => format!("invalid file content range (starts at {}, ends at {}, length {})", start, end, length),
            ParseErrorDetail::DataRangeOverflowError => format!("addition overflow while computing file content range"),
        }
    }
}

impl ParseErrorContext {
    pub fn to_msg(&self, _input: &[u8]) -> String {
        match &self {
            ParseErrorContext::PkgHeader => "reading pkg header".to_string(),
            ParseErrorContext::PkgFileList {
                entry_number,
                total_number_of_entries,
            } => format!("reading pkg file list (entry {} of {})", entry_number, total_number_of_entries),
            ParseErrorContext::PkgFileData {
                entry_number,
                total_number_of_entries,
            } => format!("reading pkg file data (entry {} of {})", entry_number, total_number_of_entries),
        }
    }
}

impl ParseError {
    pub fn to_msg(&self, input: &[u8]) -> String {
        format!("{} while {}", self.detail.to_msg(input), self.context.to_msg(input))
    }
}

pub fn load_filelist_and_raw_data(input: &[u8]) -> Result<std::vec::Vec<(FileListEntry, &[u8])>, ParseError> {
    let (number_of_files, mut ctx) = parse_pkg_header(todo!()).map_err(|e| ParseError {
        detail: e,
        context: ParseErrorContext::PkgHeader,
    })?;
    let mut filelist = std::vec::Vec::<(FileListEntry, &[u8])>::new();
    for i in 0..number_of_files {
        let (entry, entry_ctx) = parse_filelist_entry(ctx).map_err(|e| ParseError {
            detail: e,
            context: ParseErrorContext::PkgFileList {
                entry_number: i + 1,
                total_number_of_entries: number_of_files,
            },
        })?;

        let contents_start: usize = entry.data_offset.try_into().expect("32 bits please");
        let contents_end: usize = contents_start.checked_add(entry.compressed_size.try_into().expect("32 bits please")).ok_or_else(|| ParseError {
            detail: ParseErrorDetail::DataRangeOverflowError,
            context: ParseErrorContext::PkgFileData {
                entry_number: i + 1,
                total_number_of_entries: number_of_files,
            },
        })?;
        if contents_end >= input.len() {
            return Err(ParseError {
                detail: ParseErrorDetail::InvalidFileContentRange {
                    start: entry.data_offset,
                    length: entry.compressed_size,
                    end: contents_end,
                },
                context: ParseErrorContext::PkgFileData {
                    entry_number: i + 1,
                    total_number_of_entries: number_of_files,
                },
            });
        }
        filelist.push((entry, &input[contents_start..contents_end]));

        ctx = entry_ctx;
    }
    // TODO: check that data ranges are overlap-free and no data in the file is unused

    Ok(filelist)
}

fn parse_pkg_header(ctx: ParseContext) -> Result<(u32, ParseContext), ParseErrorDetail> {
    let (x, ctx) = parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)?;
    if x != 0 {
        return Err(ParseErrorDetail::UnexpectedHeaderValue { value: x });
    }
    let (number_of_files, ctx) = parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)?;
    Ok((number_of_files, ctx))
}

pub struct FileListEntry<'a> {
    filename: std::borrow::Cow<'a, str>,
    raw_size: u32,
    compressed_size: u32,
    data_offset: u32,
    compression: Compression,
}

fn parse_filelist_entry(ctx: ParseContext) -> Result<(FileListEntry, ParseContext), ParseErrorDetail> {
    let (filename, ctx) = parse_utf8cstring_padded(ctx, 64).map_err(ParseErrorDetail::Primitive)?;
    let (raw_size, ctx) = parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (compressed_size, ctx) = parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (data_offset, ctx) = parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let (compression_value, ctx) = parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)?;
    let compression = match compression_value {
        1 => Compression::None,
        _ => return Err(ParseErrorDetail::UnknownCompressionType { value: compression_value }),
    };
    return Ok((
        FileListEntry {
            filename,
            raw_size,
            compressed_size,
            data_offset,
            compression,
        },
        ctx,
    ));
}
