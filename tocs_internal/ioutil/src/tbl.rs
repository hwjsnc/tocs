use crate::{parse_bytes, parse_le_u16, parse_le_u32, parse_utf8cstring, ParseContext};

#[derive(Clone, Debug)]
pub struct ParseError {
    pub detail: ParseErrorDetail,
    pub context: ParseErrorContext,
}

#[derive(Clone, Debug)]
pub enum ParseErrorDetail {
    Primitive(crate::PrimitiveParseError),
    SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total: u16, sum: Option<u32> },
    DuplicateHeader(String),
    // TODO:
    //EntryHeaderNotInTblHeader { missing: String },
    //EntryCountDoesNotMatchHeaderEntry { expected_from_header: u32, actual: usize },
    //EntryHeaderNotInTblHeader { name: String },
}

#[derive(Clone, Debug)]
pub enum ParseErrorContext {
    TblHeader,
    TblHeaderEntry {
        start_position: usize,
        entry_number: u32,
        total_entries: u32,
    },
    EntryHeader {
        start_position: usize,
        entry_number: u16,
        total_entries: u16,
    },
    EntryData {
        header_start_position: usize,
        header_name: String,
        entry_number: u16,
        total_entries: u16,
        // TODO: attribute
    },
    End,
}

impl ParseErrorDetail {
    pub fn to_msg(&self, input: &[u8]) -> String {
        match self {
            ParseErrorDetail::Primitive(p) => p.to_msg(input),
            ParseErrorDetail::SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total, sum } => {
                format!(
                    "sum of entries per header ({}) does not match total number of entries ({}).",
                    match sum {
                        Some(n) => n as &dyn std::fmt::Display,
                        None => &"exceeds maximum" as &dyn std::fmt::Display,
                    },
                    expected_total
                )
            }
            ParseErrorDetail::DuplicateHeader(h) => format!("Tbl header name list contains duplicate ('{h}')"),
        }
    }
}

impl ParseErrorContext {
    pub fn to_msg(&self, _input: &[u8]) -> String {
        match &self {
            ParseErrorContext::TblHeader => "reading table header".to_string(),
            ParseErrorContext::TblHeaderEntry {
                start_position,
                entry_number,
                total_entries,
            } => {
                format!("reading tbl header entry {entry_number}/{total_entries} at {start_position}")
            }
            ParseErrorContext::EntryHeader {
                start_position,
                entry_number,
                total_entries,
            } => format!("reading header of entry {entry_number}/{total_entries} starting at {start_position}"),
            ParseErrorContext::EntryData {
                header_start_position,
                header_name,
                entry_number,
                total_entries,
            } => format!("reading data for entry {entry_number}/{total_entries} (header name '{header_name}' entry starts at {header_start_position})"),
            ParseErrorContext::End => "done reading".to_string(),
        }
    }
}

impl ParseError {
    pub fn to_msg(&self, input: &[u8]) -> String {
        format!("{} while {}", self.detail.to_msg(input), self.context.to_msg(input))
    }
}

/// Parse tbl header, CS1 version - basically just the number of entries
pub fn parse_tbl_header_v1(ctx: ParseContext) -> Result<(u16, ParseContext), ParseError> {
    parse_le_u16(ctx).map_err(|e| ParseError {
        detail: ParseErrorDetail::Primitive(e),
        context: ParseErrorContext::TblHeader,
    })
}

/// Parse tbl header, CS2 onwards - number of entries, number of headers in the file, number of entries per header
#[allow(clippy::type_complexity)]
pub fn parse_tbl_header_v2(ctx: ParseContext) -> Result<(u16, std::vec::Vec<(std::borrow::Cow<'_, str>, u32)>, ParseContext), ParseError> {
    let (number_of_entries, ctx) = parse_le_u16(ctx).map_err(|e| ParseError {
        detail: ParseErrorDetail::Primitive(e),
        context: ParseErrorContext::TblHeader,
    })?;
    let (header_names_with_counts, ctx) = parse_header_names_with_count(ctx)?;
    if ctx.strict {
        let actual_number_of_entries = header_names_with_counts.iter().fold(Some(0), |sum, (_, n)| n.checked_add(sum?));
        if actual_number_of_entries != Some(number_of_entries.into()) {
            return Err(ParseError {
                detail: ParseErrorDetail::SumOfEntryNumbersDiffersFromTotalEntryNumber {
                    expected_total: number_of_entries,
                    sum: actual_number_of_entries,
                },
                context: ParseErrorContext::TblHeader,
            });
        }
        let mut unique_headers = std::collections::HashSet::<&str>::new();
        for (h, _) in header_names_with_counts.iter() {
            if unique_headers.contains(h as &str) {
                return Err(ParseError {
                    detail: ParseErrorDetail::DuplicateHeader(h.to_string()),
                    context: ParseErrorContext::TblHeader,
                });
            }
            unique_headers.insert(h as &str);
        }
    }
    Ok((number_of_entries, header_names_with_counts, ctx))
}

#[allow(clippy::type_complexity)]
fn parse_header_names_with_count(ctx: ParseContext) -> Result<(std::vec::Vec<(std::borrow::Cow<'_, str>, u32)>, ParseContext), ParseError> {
    let (number_of_headers, mut ctx) = parse_le_u32(ctx).map_err(|e| ParseError {
        detail: ParseErrorDetail::Primitive(e),
        context: ParseErrorContext::TblHeader,
    })?;
    let mut headers_with_counts = std::vec::Vec::new();
    headers_with_counts.reserve_exact(number_of_headers as usize);
    for i in 0..number_of_headers {
        let (header_name, h_ctx) = parse_utf8cstring(ctx.clone()).map_err(|e| ParseError {
            detail: ParseErrorDetail::Primitive(e),
            context: ParseErrorContext::TblHeaderEntry {
                start_position: ctx.position(),
                entry_number: i + 1,
                total_entries: number_of_headers,
            },
        })?;
        let (number_of_entries, h_ctx) = parse_le_u32(h_ctx).map_err(|e| ParseError {
            detail: ParseErrorDetail::Primitive(e),
            context: ParseErrorContext::TblHeaderEntry {
                start_position: ctx.position(),
                entry_number: i + 1,
                total_entries: number_of_headers,
            },
        })?;
        headers_with_counts.push((header_name, number_of_entries));
        ctx = h_ctx;
    }
    Ok((headers_with_counts, ctx))
}

pub trait TblEntry {
    fn new_generic(header_name: &str, data: &[u8]) -> Self;
}

/// Deserialization/parsing trait for entry data (not for higher-level structure)
pub trait Deserialize
where
    Self: Sized,
{
    fn deserialize(ctx: ParseContext) -> Result<(Self, ParseContext), ParseErrorDetail>;
}

/// Serialization trait for entry data (not for higher-level structure)
pub trait Serialize<W>
where
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error>;
}

/// Trait for the length of entry data
pub trait SerializeLength {
    fn data_length(&self) -> Option<u16>;
}

pub type EntryParseFun<E> = fn(ParseContext) -> Result<(E, ParseContext), ParseErrorDetail>;

/// Parse tbl entries using the provided functions.
///
/// Attempts to parse entries without provided parser using a generic fallback.
pub fn parse_entry<'a, E: TblEntry>(
    parsers: &std::collections::HashMap<&str, EntryParseFun<E>>,
    start_ctx: ParseContext<'a>,
    entry_number: u16,
    total_entries: u16,
) -> Result<(E, ParseContext<'a>), ParseError> {
    // header name and data length
    let (header_name, ctx) = parse_utf8cstring(start_ctx.clone()).map_err(|e| ParseError {
        detail: ParseErrorDetail::Primitive(e),
        context: ParseErrorContext::EntryHeader {
            start_position: start_ctx.position(),
            entry_number,
            total_entries,
        },
    })?;
    let (data_len, ctx) = parse_le_u16(ctx).map_err(|e| ParseError {
        detail: ParseErrorDetail::Primitive(e),
        context: ParseErrorContext::EntryHeader {
            start_position: start_ctx.position(),
            entry_number,
            total_entries,
        },
    })?;

    // If we have a specialized parser function, use it; otherwise use a generic one
    if let Some(parse_entry_data) = parsers.get(&header_name as &str) {
        let (entry, ctx) = parse_entry_data(ctx).map_err(|detail| ParseError {
            detail,
            context: ParseErrorContext::EntryData {
                header_start_position: start_ctx.position(),
                header_name: header_name.to_string(),
                entry_number,
                total_entries,
            },
        })?;
        // TODO: if strict, check data length against serialized length, if unequal return appropriate error
        Ok((entry, ctx))
    } else {
        let (data, ctx) = parse_bytes(ctx, data_len as usize).map_err(|e| ParseError {
            detail: ParseErrorDetail::Primitive(e),
            context: ParseErrorContext::EntryData {
                header_start_position: start_ctx.position(),
                header_name: header_name.to_string(),
                entry_number,
                total_entries,
            },
        })?;

        let entry = E::new_generic(&header_name as &str, data);
        Ok((entry, ctx))
    }
}

/// Parse all tbl entries using the provided functions.
///
/// Attempts to parse entries without provided parser using a generic fallback.
pub fn parse_entries<'a, E: TblEntry>(
    parsers: &std::collections::HashMap<&str, EntryParseFun<E>>,
    mut ctx: ParseContext<'a>,
    number_of_entries: u16,
) -> Result<(std::vec::Vec<E>, ParseContext<'a>), ParseError> {
    let mut entries = std::vec::Vec::new();
    entries.reserve_exact(number_of_entries as usize);
    for i in 0..number_of_entries {
        let (entry, entry_ctx) = parse_entry(parsers, ctx, i + 1, number_of_entries)?;
        entries.push(entry);
        ctx = entry_ctx;
    }
    Ok((entries, ctx))
}

impl Deserialize for u8 {
    fn deserialize(ctx: ParseContext) -> Result<(Self, ParseContext), ParseErrorDetail> {
        crate::parse_u8(ctx).map_err(ParseErrorDetail::Primitive)
    }
}

impl Deserialize for u16 {
    fn deserialize(ctx: ParseContext) -> Result<(Self, ParseContext), ParseErrorDetail> {
        crate::parse_le_u16(ctx).map_err(ParseErrorDetail::Primitive)
    }
}

impl Deserialize for u32 {
    fn deserialize(ctx: ParseContext) -> Result<(Self, ParseContext), ParseErrorDetail> {
        crate::parse_le_u32(ctx).map_err(ParseErrorDetail::Primitive)
    }
}

impl Deserialize for i16 {
    fn deserialize(ctx: ParseContext) -> Result<(Self, ParseContext), ParseErrorDetail> {
        crate::parse_le_i16(ctx).map_err(ParseErrorDetail::Primitive)
    }
}

impl Deserialize for i32 {
    fn deserialize(ctx: ParseContext) -> Result<(Self, ParseContext), ParseErrorDetail> {
        crate::parse_le_i32(ctx).map_err(ParseErrorDetail::Primitive)
    }
}

impl Deserialize for String {
    fn deserialize(ctx: ParseContext) -> Result<(Self, ParseContext), ParseErrorDetail> {
        crate::parse_utf8cstring(ctx).map(|(s, ctx)| (s.into_owned(), ctx)).map_err(ParseErrorDetail::Primitive)
    }
}

impl<W> Serialize<W> for u8
where
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        out.write(&[*self]).map(|_| ())
    }
}

impl<W> Serialize<W> for u16
where
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        out.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl<W> Serialize<W> for u32
where
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        out.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl<W> Serialize<W> for i16
where
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        out.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl<W> Serialize<W> for i32
where
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        out.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl<W> Serialize<W> for f32
where
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        out.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl<W> Serialize<W> for str
where
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        crate::serialize_utf8cstring(out, self)
    }
}

impl<W, T, const N: usize> Serialize<W> for &[T; N]
where
    T: Serialize<W>,
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        for x in self.iter() {
            x.serialize(out)?;
        }
        Ok(())
    }
}

impl<W, T, const N: usize> Serialize<W> for [T; N]
where
    T: Serialize<W>,
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        for x in self.iter() {
            x.serialize(out)?;
        }
        Ok(())
    }
}

impl<W, T> Serialize<W> for std::vec::Vec<T>
where
    T: Serialize<W>,
    W: std::io::Write,
{
    fn serialize(&self, out: &mut W) -> Result<(), std::io::Error> {
        for x in self.iter() {
            x.serialize(out)?;
        }
        Ok(())
    }
}

impl SerializeLength for u8 {
    fn data_length(&self) -> Option<u16> {
        std::mem::size_of::<Self>().try_into().ok()
    }
}

impl SerializeLength for u16 {
    fn data_length(&self) -> Option<u16> {
        std::mem::size_of::<Self>().try_into().ok()
    }
}

impl SerializeLength for u32 {
    fn data_length(&self) -> Option<u16> {
        std::mem::size_of::<Self>().try_into().ok()
    }
}

impl SerializeLength for i16 {
    fn data_length(&self) -> Option<u16> {
        std::mem::size_of::<Self>().try_into().ok()
    }
}

impl SerializeLength for i32 {
    fn data_length(&self) -> Option<u16> {
        std::mem::size_of::<Self>().try_into().ok()
    }
}

impl SerializeLength for f32 {
    fn data_length(&self) -> Option<u16> {
        std::mem::size_of::<Self>().try_into().ok()
    }
}

impl SerializeLength for str {
    fn data_length(&self) -> Option<u16> {
        1_u16.checked_add(self.len().try_into().ok()?)
    }
}

impl<T: SerializeLength, const N: usize> SerializeLength for [T; N] {
    fn data_length(&self) -> Option<u16> {
        let mut sum: Option<u16> = Some(0);
        for x in self.iter() {
            sum = sum?.checked_add(x.data_length()?);
        }
        sum
    }
}

impl<T: SerializeLength> SerializeLength for std::vec::Vec<T> {
    fn data_length(&self) -> Option<u16> {
        let mut sum: Option<u16> = Some(0);
        for x in self.iter() {
            sum = sum?.checked_add(x.data_length()?);
        }
        sum
    }
}
