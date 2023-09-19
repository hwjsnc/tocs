use super::{PaddedAsciiStr, PaddedByteStr, PaddedUtf8Str};

pub struct MemWriter {
    inner: std::vec::Vec<u8>,
}

impl MemWriter {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(cap: usize) -> Self {
        MemWriter {
            inner: std::vec::Vec::with_capacity(cap),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn into_inner(self) -> std::vec::Vec<u8> {
        let MemWriter { inner } = self;
        inner
    }
}

macro_rules! write_primitive {
    ($f:ident, $t:ty) => {
        #[inline(always)]
        pub fn $f(&mut self, x: $t) {
            self.inner.extend_from_slice(&x.to_le_bytes());
        }
    };
}

impl MemWriter {
    write_primitive! {write_u8, u8}
    write_primitive! {write_i8, i8}
    write_primitive! {write_u16_le, u16}
    write_primitive! {write_u32_le, u32}
    write_primitive! {write_i16_le, i16}
    write_primitive! {write_i32_le, i32}
    write_primitive! {write_f32_le, f32}
}

impl MemWriter {
    pub fn write_data(&mut self, s: &[u8]) {
        self.inner.extend_from_slice(s);
    }

    pub fn write_padded_byte_str<const PADDING: usize>(&mut self, s: PaddedByteStr<'_, PADDING>) {
        s.write_to(&mut self.inner).unwrap(); // writing to vec should not fail
    }

    #[allow(dead_code)]
    pub fn write_padded_ascii_str<const PADDING: usize>(&mut self, s: PaddedAsciiStr<'_, PADDING>) {
        s.write_to(&mut self.inner).unwrap();
    }

    #[allow(dead_code)]
    pub fn write_padded_utf8_str<const PADDING: usize>(&mut self, s: PaddedUtf8Str<'_, PADDING>) {
        s.write_to(&mut self.inner).unwrap();
    }
}
