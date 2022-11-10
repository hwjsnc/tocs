#[derive(Clone, Copy, Debug)]
pub enum Compression {
    None,
    LZSS,
    LZ4,
    Zstd,
    Unknown(u32),
}

impl std::fmt::Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Compression::None => write!(f, "uncompressed"),
            Compression::LZSS => write!(f, "LZSS"),
            Compression::LZ4 => write!(f, "LZ4"),
            Compression::Zstd => write!(f, "zstd"),
            Compression::Unknown(x) => write!(f, "unknown compression type (value x (0x{x:x}"),
        }
    }
}

impl std::convert::From<u32> for Compression {
    fn from(x: u32) -> Compression {
        match x {
            0 => Compression::None,
            1 => Compression::LZSS,
            4 => Compression::LZ4,
            16 => Compression::Zstd,
            _ => Compression::Unknown(x),
        }
    }
}

impl Compression {
    pub fn decompress_to_buffer(&self, r#in: &[u8], out: &mut [u8]) -> Result<(), String> {
        let written = match self {
            Compression::None => copy_buffer(r#in, out),
            Compression::LZSS => lzss_decompress_to_buffer(r#in, out),
            Compression::LZ4 => lz4::block::decompress_to_buffer(r#in, None, out).map_err(|e| format!("lz4 decompression error: {e}")),
            Compression::Zstd => zstd::bulk::decompress_to_buffer(r#in, out).map_err(|e| format!("zstd decompression error: {e}")),
            Compression::Unknown(x) => Err(format!("unsupported/unknown compression type {x}")),
        }?;
        if written != out.len() {
            return Err(format!("{} decompression error: decompressed size ({}) differs from expected size ({})", written, written, r#out.len()));
        }
        Ok(())
    }

    pub fn decompress_with_exact_output_size(&self, r#in: &[u8], len: usize) -> Result<std::vec::Vec<u8>, String> {
        let mut out = vec![0; len];
        self.decompress_to_buffer(r#in, &mut out)?;
        Ok(out)
    }
}

fn copy_buffer(r#in: &[u8], out: &mut [u8]) -> Result<usize, String> {
    if r#in.len() != out.len() {
        return Err(format!(
            "uncompressed error: decompressed size mismatch ({} = 0x{:x} versus {} = 0x{:x})",
            r#in.len(),
            r#in.len(),
            out.len(),
            out.len()
        ));
    }
    out.copy_from_slice(r#in);
    Ok(r#in.len())
}

fn lzss_decompress_to_buffer(r#in: &[u8], out: &mut [u8]) -> Result<usize, String> {
    if r#in.len() < 12 {
        return Err(format!("lzss error: compressed size ({}) is too small for lzss header", r#in.len()));
    }

    let decompressed_size = u32::from_le_bytes(r#in[0..4].try_into().expect("length should be >= 12"));
    let compressed_size = u32::from_le_bytes(r#in[4..8].try_into().expect("length should be >= 12"));
    let key = u32::from_le_bytes(r#in[8..12].try_into().expect("length should be >= 12"));
    let key: u8 = key.try_into().map_err(|_| String::from("lzss error: invalid compression key > 255"))?;

    if <u32 as TryInto<usize>>::try_into(decompressed_size).expect("32 bits please") != r#out.len() {
        return Err(format!(
            "lzss error: decompressed size mismatch ({} = 0x{:x} versus {} = 0x{:x})",
            decompressed_size,
            decompressed_size,
            r#in.len(),
            r#in.len()
        ));
    }
    if <u32 as TryInto<usize>>::try_into(compressed_size).expect("32 bits please") != r#in.len() {
        return Err(format!(
            "lzss error: compressed size mismatch ({} = 0x{:x} versus {} = 0x{:x})",
            compressed_size,
            compressed_size,
            r#out.len(),
            r#out.len()
        ));
    }

    let mut in_idx = 12;
    let mut out_idx = 0;
    while in_idx < r#in.len() && out_idx <= out.len() {
        let byte = r#in[in_idx];
        in_idx += 1;

        if byte == key {
            log::trace!("chunk");
            let off = r#in.get(in_idx).copied().ok_or_else(|| String::from("chunk offset past end of compressed data"))?;
            in_idx += 1;
            if off == key {
                out[out_idx] = key;
                out_idx += 1;
            } else {
                let off: usize = if off > key { off - 1 } else { off }.into();
                let len: usize = r#in.get(in_idx).copied().ok_or_else(|| String::from("chunk length past end of compressed data"))?.into();
                in_idx += 1;
                let copy_start = out_idx
                    .checked_sub(off)
                    .ok_or_else(|| String::from("lzss error: decompression offset must be <= length of decompressed data"))?;
                for i in 0..len {
                    out[out_idx + i] = out[copy_start + i];
                }
                out_idx += len;
            }
        } else {
            out[out_idx] = byte;
            out_idx += 1;
        }
    }

    assert_eq!(in_idx, r#in.len());
    assert_eq!(out_idx, out.len());

    Ok(out_idx)
}
