#[derive(Clone)]
pub struct MemReader<'a> {
    data: &'a [u8],
    /// Invariant: off <= data.len()
    off: usize,
}

impl<'a> MemReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, off: 0 }
    }

    pub fn offset(&self) -> usize {
        self.off
    }

    pub fn set_offset(&mut self, off: usize) {
        assert!(off <= self.data.len());
        self.off = off;
    }

    pub fn at_end(&self) -> bool {
        self.off == self.data.len()
    }

    pub fn rest(&self) -> &'a [u8] {
        &self.data[self.off..]
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let result = self.data.get(self.off).copied()?;
        self.off += 1;
        Some(result)
    }

    pub fn read_chunk<const N: usize>(&mut self) -> Option<[u8; N]> {
        let result: [u8; N] = self.data.get(self.off..self.off + N)?.try_into().ok()?;
        self.off += N;
        Some(result)
    }

    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        let result = self.data.get(self.off..self.off + n)?;
        self.off += n;
        Some(result)
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        self.read_byte()
    }

    pub fn read_i8(&mut self) -> Option<i8> {
        self.read_chunk::<1>().map(i8::from_le_bytes)
    }

    pub fn read_u16_le(&mut self) -> Option<u16> {
        self.read_chunk::<2>().map(u16::from_le_bytes)
    }

    pub fn read_i16_le(&mut self) -> Option<i16> {
        self.read_chunk::<2>().map(i16::from_le_bytes)
    }

    pub fn read_u32_le(&mut self) -> Option<u32> {
        self.read_chunk::<4>().map(u32::from_le_bytes)
    }

    pub fn read_i32_le(&mut self) -> Option<i32> {
        self.read_chunk::<4>().map(i32::from_le_bytes)
    }

    pub fn read_f32_le(&mut self) -> Option<f32> {
        self.read_chunk::<4>().map(f32::from_le_bytes)
    }

    pub fn read_padded_byte_str<const PADDING: usize>(&mut self) -> Result<super::PaddedByteStr<'a, PADDING>, super::PaddedStringReadError> {
        let (bytes, _rest) = super::PaddedByteStr::<'_, PADDING>::read_from_slice(self.rest())?;
        self.off += bytes.len_with_padding().get();
        debug_assert!(self.off <= self.data.len());
        Ok(bytes)
    }

    pub fn read_padded_ascii_str<const PADDING: usize>(&mut self) -> Result<super::PaddedAsciiStr<'a, PADDING>, super::PaddedStringReadError> {
        let (ascii, _rest) = super::PaddedAsciiStr::<'_, PADDING>::read_from_slice(self.rest())?;
        self.off += ascii.len_with_padding().get();
        debug_assert!(self.off <= self.data.len());
        Ok(ascii)
    }

    pub fn read_padded_utf8_str<const PADDING: usize>(&mut self) -> Result<super::PaddedUtf8Str<'a, PADDING>, super::PaddedStringReadError> {
        let (utf8, _rest) = super::PaddedUtf8Str::<'_, PADDING>::read_from_slice(self.rest())?;
        self.off += utf8.len_with_padding().get();
        debug_assert!(self.off <= self.data.len());
        Ok(utf8)
    }

    pub fn read_padded_utf8_string_lossy<const PADDING: usize>(&mut self) -> Result<super::PaddedUtf8String<PADDING>, super::PaddedStringReadError> {
        let (bytes, _rest) = super::PaddedByteStr::<'a, PADDING>::read_from_slice(self.rest())?;
        self.off += bytes.len_with_padding().get();
        debug_assert!(self.off <= self.data.len());
        Ok(super::PaddedUtf8String::from_utf8_lossy(bytes))
    }

    pub fn read_c_byte_str(&mut self) -> Result<super::PaddedByteStr<'a, 1>, super::PaddedStringReadError> {
        self.read_padded_byte_str::<1>()
    }

    pub fn read_c_ascii_str(&mut self) -> Result<super::PaddedAsciiStr<'a, 1>, super::PaddedStringReadError> {
        self.read_padded_ascii_str::<1>()
    }

    pub fn read_c_utf8_str(&mut self) -> Result<super::PaddedUtf8Str<'a, 1>, super::PaddedStringReadError> {
        self.read_padded_utf8_str::<1>()
    }

    pub fn read_c_utf8_string_lossy(&mut self) -> Result<super::PaddedUtf8String<1>, super::PaddedStringReadError> {
        self.read_padded_utf8_string_lossy::<1>()
    }
}
