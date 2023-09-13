// TODO: I'm deserializing entries as vec<(header, indexmap<attr, val>)> here because that's how I did it before and I don't care to change the API _again_.
// However, it would probably be a better idea to deserialize as (&Schema, Vec<val>).
// The downside to this is the awful lifetime management in programs with long-lived `Tbl`s.

use crate::io::{MemReader, PaddedAsciiStr, PaddedByteStr, PaddedUtf8Str, PaddedUtf8String};

use super::{AtomicSchemas, AtomicType, AtomicValue, Attribute, Entry, Header, Version, VersionWithoutSchema};
use crate::tbl::schema::HeaderRef;

use indexmap::IndexMap;

#[derive(Clone, Debug)]
pub enum DeserializeErrorDetail {
    Eof,
    Ascii,
    Utf8(std::str::Utf8Error, ecow::EcoString),
    SumOfEntryNumbersDiffersFromTotalEntryNumber { expected_total: u16, sum: Option<u32> },
    DuplicateHeader(Header),
    EntryHeaderNotInTblHeader { missing: Header },
    EntryCountDoesNotMatchHeaderEntry { header: Header, expected: u32, actual: usize },
    EntryDataLengthMismatch { expected: u16, actual: Option<u16> },
    UnexpectedNumberOfEntries { expected: u16, actual: usize },
    TrailingData,
}

#[derive(Clone, Debug)]
pub enum DeserializeErrorPosition {
    // entry numbers should start at 0 here but will be reported as starting with 1
    Start,
    TblHeader,
    TblHeaderEntry { entry_number: u32 },
    Entry { number: u16 },
    EntryHeader,
    EntryData,
    HeaderName,
    Value(AtomicType),
    Attribute(Attribute),
    End,
}

#[derive(Clone, Debug)]
pub struct DeserializeError {
    pub detail: DeserializeErrorDetail,
    /// Seek position in the file
    pub position: usize,
    /// Last element is the position of the original cause, the rest is how we got there.
    ///
    /// May be empty.
    pub context: std::vec::Vec<DeserializeErrorPosition>,
}

pub struct TblReaderAtomic<'a> {
    inner: MemReader<'a>,
    version: VersionWithoutSchema,
    /// Warning log
    ///
    /// If `None`, no warnings are emitted and parsing is strict.
    warning_log: Option<&'a mut std::vec::Vec<DeserializeError>>,
    /// Context stack (what we are working on right now)
    ///
    /// The most specific position is at the end
    context: std::vec::Vec<DeserializeErrorPosition>, // TODO: option to not build context stack?
    schemas: &'a AtomicSchemas,
}

impl<'a> TblReaderAtomic<'a> {
    pub fn new(data: &'a [u8], version: &'a Version) -> Self {
        Self {
            version: version.without_schema(),
            inner: MemReader::new(data),
            warning_log: None,
            context: std::vec::Vec::with_capacity(5),
            schemas: version.schemas(),
        }
    }

    pub fn new_lax(data: &'a [u8], version: &'a Version, warning_log: &'a mut std::vec::Vec<DeserializeError>) -> Self {
        let mut reader = Self::new(data, version);
        reader.warning_log = Some(warning_log);
        reader
    }
}

impl<'a> TblReaderAtomic<'a> {
    /// Create an error in the current context, and log it if possible.
    pub fn error(&mut self, detail: DeserializeErrorDetail) -> DeserializeError {
        let err = DeserializeError {
            detail,
            position: self.inner.offset(),
            context: self.context.clone(),
        };
        if let Some(log) = self.warning_log.as_mut() {
            log.push(err.clone());
        }
        err
    }

    pub fn warning(&mut self, detail: DeserializeErrorDetail) {
        _ = self.error(detail);
    }

    pub fn enter_context(&mut self, pos: DeserializeErrorPosition) {
        self.context.push(pos);
    }

    pub fn exit_context(&mut self) {
        self.context.pop().unwrap();
    }

    /// Add the given position to the context, run the provided closure, then remove the context again and return the closure return value
    pub fn in_context<T, F>(&mut self, ctx: DeserializeErrorPosition, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.enter_context(ctx);
        let ret = f(self);
        self.exit_context();
        ret
    }

    pub fn at_end(&self) -> bool {
        self.inner.at_end()
    }

    #[inline(always)]
    fn strict(&self) -> bool {
        self.warning_log.is_none()
    }

    #[inline(always)]
    fn non_strict(&self) -> bool {
        !self.strict()
    }
}

macro_rules! impl_primitive_read_via_inner {
    ($f:ident, $t:ty, $at:expr) => {
        pub fn $f(&mut self) -> Result<$t, DeserializeError> {
            self.in_context(
                DeserializeErrorPosition::Value($at),
                |this| this.inner.$f().ok_or_else(|| this.error(DeserializeErrorDetail::Eof)), // EOF is the only way reading a primitive can fail
            )
        }
    };
}

impl<'a> TblReaderAtomic<'a> {
    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], DeserializeError> {
        self.in_context(DeserializeErrorPosition::Value(AtomicType::X(n)), |this| {
            this.inner.read_bytes(n).ok_or_else(|| this.error(DeserializeErrorDetail::Eof))
        })
    }

    impl_primitive_read_via_inner! {read_byte, u8, AtomicType::U8}
    impl_primitive_read_via_inner! {read_u8, u8, AtomicType::U8}
    impl_primitive_read_via_inner! {read_i8, i8, AtomicType::I8}

    impl_primitive_read_via_inner! {read_u16_le, u16, AtomicType::U16}
    impl_primitive_read_via_inner! {read_u32_le, u32, AtomicType::U32}
    impl_primitive_read_via_inner! {read_i16_le, i16, AtomicType::I16}
    impl_primitive_read_via_inner! {read_i32_le, i32, AtomicType::I32}
    impl_primitive_read_via_inner! {read_f32_le, f32, AtomicType::F32}

    /// Read c byte string.
    ///
    /// Does not modify parser context, that's the caller's job.
    fn _read_c_byte_str(&mut self) -> Result<PaddedByteStr<'a, 1>, DeserializeError> {
        use crate::io::PaddedStringReadError;

        self.inner.read_c_byte_str().map_err(|err| {
            self.error(match err {
                PaddedStringReadError::Io(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => DeserializeErrorDetail::Eof,
                PaddedStringReadError::Io(_) => unreachable!(),
                PaddedStringReadError::Padding(_) => unreachable!(), // tbls don't have padded strings, only null-terminated
                PaddedStringReadError::Ascii | PaddedStringReadError::Utf8(_) => unreachable!(), // we're not reading ASCII or UTF-8
            })
        })
    }

    pub fn read_c_utf8_str(&mut self) -> Result<PaddedUtf8Str<'a, 1>, DeserializeError> {
        use crate::io::PaddedStringError;

        self.in_context(DeserializeErrorPosition::Value(AtomicType::CUtf8), |this| {
            let bytes = this._read_c_byte_str()?;
            bytes.try_into().map_err(|err| match err {
                PaddedStringError::Utf8(err) => {
                    let s = String::from_utf8_lossy(bytes.as_bytes_excluding_terminator_and_padding()).into();
                    this.error(DeserializeErrorDetail::Utf8(err, s))
                }
                PaddedStringError::ContainsNull => unreachable!(),
                PaddedStringError::Ascii => unreachable!(),
            })
        })
    }

    pub fn read_header_name(&mut self) -> Result<HeaderRef<'a>, DeserializeError> {
        use crate::io::PaddedStringError;

        self.in_context(DeserializeErrorPosition::HeaderName, |this| {
            let bytes = this._read_c_byte_str()?;

            PaddedAsciiStr::<'_, 1>::try_from(bytes).map(HeaderRef::from).map_err(|err| match err {
                PaddedStringError::Utf8(err) => {
                    let s = String::from_utf8_lossy(bytes.as_bytes_excluding_terminator_and_padding()).into();
                    this.error(DeserializeErrorDetail::Utf8(err, s))
                }
                PaddedStringError::ContainsNull => unreachable!(),
                PaddedStringError::Ascii => this.error(DeserializeErrorDetail::Ascii),
            })
        })
    }
}

impl<'a> TblReaderAtomic<'a> {
    /// Read tbl header for Trails of Cold Steel (no list of entry headers at the start)
    pub fn read_tbl_header_v1(&mut self) -> Result<u16, DeserializeError> {
        self.in_context(DeserializeErrorPosition::TblHeader, |this| this.read_u16_le())
    }

    /// Read tbl header for Trails of Cold Steel 2 - 5 (includes list of entry headers at the start)
    pub fn read_tbl_header_v2(&mut self) -> Result<(u16, IndexMap<Header, u32>), DeserializeError> {
        self.in_context(DeserializeErrorPosition::TblHeader, |this| {
            let number_of_entries = this.read_u16_le()?;
            let number_of_headers = this.read_u32_le()?;
            let mut entry_counts = IndexMap::with_capacity(number_of_entries.try_into().unwrap());
            for i in 0..number_of_headers {
                this.in_context(DeserializeErrorPosition::TblHeaderEntry { entry_number: i }, |this| {
                    let hdr = this.read_header_name()?.to_owned(); // TODO: clone an existing known header (if there is one)
                    let count = this.read_u32_le()?;
                    match entry_counts.entry(hdr) {
                        indexmap::map::Entry::Vacant(vacant) => {
                            vacant.insert(count);
                        }
                        indexmap::map::Entry::Occupied(previous) => {
                            let (hdr, _) = previous.remove_entry();
                            return Err(this.error(DeserializeErrorDetail::DuplicateHeader(hdr)));
                        }
                    }
                    Ok(())
                })?;
            }

            // check counts
            let sum_of_counts: Option<u32> = entry_counts.values().try_fold(0u32, |a, b| a.checked_add(*b));
            if sum_of_counts != Some(number_of_entries.into()) {
                let err = this.error(DeserializeErrorDetail::SumOfEntryNumbersDiffersFromTotalEntryNumber {
                    expected_total: number_of_entries,
                    sum: sum_of_counts,
                });
                if this.strict() {
                    return Err(err);
                }
            }

            Ok((number_of_entries, entry_counts))
        })
    }

    pub fn read_value(&mut self, t: AtomicType) -> Result<AtomicValue, DeserializeError> {
        match t {
            AtomicType::I8 => self.read_i8().map(AtomicValue::I8),
            AtomicType::I16 => self.read_i16_le().map(AtomicValue::I16),
            AtomicType::I32 => self.read_i32_le().map(AtomicValue::I32),
            AtomicType::U8 => self.read_u8().map(AtomicValue::U8),
            AtomicType::U16 => self.read_u16_le().map(AtomicValue::U16),
            AtomicType::U32 => self.read_u32_le().map(AtomicValue::U32),
            AtomicType::F32 => self.read_f32_le().map(AtomicValue::F32),
            AtomicType::CUtf8 => match self.read_c_utf8_str() {
                Ok(utf8) => Ok(AtomicValue::CUtf8(utf8.as_str().into())),
                Err(DeserializeError {
                    detail: DeserializeErrorDetail::Utf8(_, s),
                    ..
                }) if self.non_strict() => Ok(AtomicValue::CUtf8(s)),
                Err(err) => Err(err),
            },
            AtomicType::X(n) => self.read_bytes(n).map(|slice| AtomicValue::X(slice.into())),
        }
    }

    pub fn read_tbl_entry_data(&mut self, schema: &IndexMap<Attribute, AtomicType>) -> Result<IndexMap<Attribute, AtomicValue>, DeserializeError> {
        self.in_context(DeserializeErrorPosition::EntryData, |this| {
            let mut values = IndexMap::with_capacity(schema.len());
            for (i, (attr_name, attr_type)) in schema.iter().enumerate() {
                let val = this.in_context(DeserializeErrorPosition::Attribute(attr_name.clone()), |this| this.read_value(*attr_type))?;
                let previous = values.insert(attr_name.clone(), val);
                assert!(previous.is_none());
            }
            Ok(values)
        })
    }

    /// Read entry data as a blob for an entry with unknown schema
    pub fn read_generic_tbl_entry_data(&mut self, len: usize) -> Result<IndexMap<Attribute, AtomicValue>, DeserializeError> {
        self.in_context(DeserializeErrorPosition::EntryData, |this| {
            let attr_name = Attribute::from("data");
            let data = this.in_context(DeserializeErrorPosition::Attribute(attr_name.clone()), |this| this.read_value(AtomicType::X(len)))?;
            Ok([(attr_name, data)].into_iter().collect())
        })
    }

    fn _read_entry(&mut self) -> Result<Entry, DeserializeError> {
        // read header and data
        let (header_ref, data_len) = self.in_context(DeserializeErrorPosition::EntryHeader, |this| {
            let header_ref = this.read_header_name()?;
            let data_len = this.read_u16_le()?;
            Ok((header_ref, data_len))
        })?;
        let data = match self.schemas.get(&header_ref) {
            // TODO: special case for Reverie NameTableData
            Some(schema) => self.read_tbl_entry_data(&schema)?,
            None => self.read_generic_tbl_entry_data(data_len.into())?,
        };

        // check actual data length against what we read
        let actual_len = super::ser::atomic_attributes_serialization_length(&data);
        if actual_len != Some(data_len) {
            let err = self.error(DeserializeErrorDetail::EntryDataLengthMismatch {
                expected: data_len,
                actual: actual_len,
            });
            if self.strict() {
                return Err(err);
            }
        }
        todo!()
    }

    pub fn read_entries(&mut self, expected_total_count: u16, expected_counts: Option<&IndexMap<Header, u32>>) -> Result<std::vec::Vec<Entry>, DeserializeError> {
        let mut entries = std::vec::Vec::with_capacity(expected_total_count.into());

        loop {
            if self.at_end() || (self.strict() && entries.len() >= usize::from(expected_total_count)) {
                break;
            }

            self.in_context(
                DeserializeErrorPosition::Entry {
                    number: entries.len().try_into().unwrap_or(u16::MAX),
                },
                |this| {
                    // read entry
                    let entry = this._read_entry()?;

                    // check that entry header is present in tbl header
                    if let Some(counts) = expected_counts.as_ref() {
                        if !counts.contains_key(&entry.header) {
                            let missing = entry.header.clone();
                            let err = this.in_context(DeserializeErrorPosition::EntryHeader, |this| this.error(DeserializeErrorDetail::EntryHeaderNotInTblHeader { missing }));
                            if this.strict() {
                                return Err(err);
                            }
                        }
                    }

                    entries.push(entry);

                    Ok(())
                },
            )?;
        }

        self.in_context(DeserializeErrorPosition::End, move |this| {
            // We should have reached the end
            if !this.at_end() {
                return Err(this.error(DeserializeErrorDetail::TrailingData));
            }

            // Did we read too many or too few entries?
            if entries.len() != usize::from(expected_total_count) {
                let err = this.error(DeserializeErrorDetail::UnexpectedNumberOfEntries {
                    expected: expected_total_count,
                    actual: entries.len(),
                });
                if this.strict() {
                    return Err(err);
                }
            }

            // Check entry counts per header
            if let Some(counts) = expected_counts {
                // Note: we don't need to count headers that aren't expected since we already reported them as missing from the tbl header.
                for (header, count) in counts.iter() {
                    // O(n^2) but w/e
                    let actual = entries.iter().filter(|ent| &ent.header == header).count();
                    if actual != usize::try_from(*count).unwrap() {
                        let err = this.error(DeserializeErrorDetail::EntryCountDoesNotMatchHeaderEntry {
                            header: header.clone(),
                            expected: *count,
                            actual,
                        });
                        if this.strict() {
                            return Err(err);
                        }
                    }
                }
            }

            Ok(entries)
        })
    }

    pub fn read(&mut self) -> Result<super::Tbl, DeserializeError> {
        match self.version {
            VersionWithoutSchema::V1 => {
                let entry_count = self.read_tbl_header_v1()?;
                let entries = self.read_entries(entry_count, None)?;
                let headers = smallvec::SmallVec::new();
                Ok(super::Tbl::new_2(self.version, headers, entries))
            }
            VersionWithoutSchema::V2 => {
                let (total_count, entry_counts) = self.read_tbl_header_v2()?;
                let entries = self.read_entries(total_count, Some(&entry_counts))?;
                let headers = entry_counts.into_iter().map(|(h, _)| h).collect();
                Ok(super::Tbl::new_2(self.version, headers, entries))
            }
        }
    }
}
