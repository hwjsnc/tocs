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
            Compression::LZ4 => {
                let len: i32 = out.len().try_into().map_err(|_| format!("lz4 decompression error: output size too big"))?;
                lz4::block::decompress_to_buffer(r#in, Some(len), out).map_err(|e| format!("lz4 decompression error: {e}"))
            }
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

    pub fn compress(&self, r#in: &[u8]) -> Result<std::vec::Vec<u8>, String> {
        match self {
            Compression::None => Ok(r#in.to_vec()),
            Compression::LZSS => lzss_compress(r#in).map_err(|e| format!("lzss compression error: {e}")),
            Compression::LZ4 => {
                // high compression mode 12 is the maximum.
                // For most of the .pkgs in the CS3 PC port if you decompress and recompress the data with this setting, you will get a bit-exact replica of the original compressed data.
                // (There is one file for which this isn't true, I guess this might be down to a different compressor version.)
                let mode = Some(lz4::block::CompressionMode::HIGHCOMPRESSION(12));
                lz4::block::compress(r#in, mode, false).map_err(|e| format!("lz4 compression error: {e}"))
            }
            Compression::Zstd => {
                let level = 15; // This compression level approximately matches the one used in the switch port of CS4.
                                // I wasn't able to get bit-exact compatibility of the compressed data for any setting (probably a different compressor used?).
                zstd::bulk::compress(r#in, level).map_err(|e| format!("zstd compression error: {e}"))
            }
            Compression::Unknown(x) => Err(format!("unsupported/unknown compression type {x}")),
        }
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
            r#out.len(),
            r#out.len()
        ));
    }
    if <u32 as TryInto<usize>>::try_into(compressed_size).expect("32 bits please") != r#in.len() {
        return Err(format!(
            "lzss error: compressed size mismatch ({} = 0x{:x} versus {} = 0x{:x})",
            compressed_size,
            compressed_size,
            r#in.len(),
            r#in.len()
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
                if len < off {
                    let (previous, next) = out.split_at_mut(out_idx);
                    next[0..len].copy_from_slice(&previous[copy_start..copy_start+len]);
                } else {
                    for i in 0..len {
                        out[out_idx + i] = out[copy_start + i];
                    }
                }
                out_idx += len;
            }
        } else {
            out[out_idx] = byte;
            out_idx += 1;
        }
    }

    assert_eq!(in_idx, r#in.len());

    if out_idx != out.len() {
        return Err(format!("size of decompressed data ({}) does not match expected size ({})", out_idx, out.len()));
    }

    Ok(out_idx)
}

fn lzss_compress(data: &[u8]) -> Result<std::vec::Vec<u8>, String> {
    // find the most rare byte, choose it as key value
    let key = most_rare_byte(data);
    lzss_compress_with_key(data, key)
}

fn lzss_compress_with_key(data: &[u8], key: u8) -> Result<std::vec::Vec<u8>, String> {
    let mut output = vec![];
    output.extend_from_slice(
        &<usize as TryInto<u32>>::try_into(data.len())
            .map_err(|_| String::from("input size exceeds 32 bit maximum"))?
            .to_le_bytes(),
    ); // uncompressed size
    output.extend_from_slice(&[0, 0, 0, 0]); // compressed size, we fix this later
    output.extend_from_slice(&<u8 as Into<u32>>::into(key).to_le_bytes()); // compression key

    let mut i = 0;
    while i < data.len() {
        // find longest prefix of the remaining input in the buffer of previous bytes
        let ref_buf = &data[i.saturating_sub(254)..i];
        let (ref_off, ref_len) = longest_prefix_in(&data[i..], ref_buf);
        assert!(ref_off + ref_len <= ref_buf.len());
        if ref_len > 3 {
            // inserting a reference takes 3 bytes, so the reference has to be longer than that if we want to compress
            let off = ref_buf.len().checked_sub(ref_off).expect("ref_off should be < ref_buf.len()"); // offset should be from the end, but we got it from the beginning of the buffer
            assert!(off >= ref_len);
            let off = if off >= key.into() { off + 1 } else { off };
            let off: u8 = off.try_into().expect("offset should be <= 255");
            let len: u8 = ref_len.try_into().expect("ref_len should be <= off <= 255");
            output.extend_from_slice(&[key, off, len]);
            i += ref_len;
        } else if data[i] == key {
            output.extend_from_slice(&[key, key]);
            i += 1;
        } else {
            output.push(data[i]);
            i += 1;
        }
    }

    let compressed_size: u32 = output.len().try_into().map_err(|_| String::from("output size exceeds 32 bit maximum"))?;
    output[4..8].copy_from_slice(&compressed_size.to_le_bytes());

    Ok(output)
}

fn most_rare_byte(s: &[u8]) -> u8 {
    // how often does each byte occur in s?
    let mut counts = [0; 256];
    for x in s {
        let x: usize = (*x).into();
        counts[x] += 1;
    }
    // which one has the fewest counts?
    let mut argmin = 0;
    for i in 0..counts.len() {
        if counts[i] < counts[argmin] {
            argmin = i;
        }
    }
    argmin.try_into().expect("array indices are <= 255")
}

fn length_of_common_prefix(s1: &[u8], s2: &[u8]) -> usize {
    for i in 0..std::cmp::min(s1.len(), s2.len()) {
        if s1[i] != s2[i] {
            return i;
        }
    }
    return std::cmp::min(s1.len(), s2.len());
}

/// Given two strings s1, s2, find the longest prefix of s1 that occurs in s2.
///
/// Returns the offset of the prefix in s2 and its length (may be 0).
fn longest_prefix_in(s1: &[u8], s2: &[u8]) -> (usize, usize) {
    let mut off = 0; // offset of the substring in s1
    let mut len = 0; // length of the substring
    let mut i = 0; // current position in s1
    while i + len < s2.len() {
        let x = length_of_common_prefix(s1, &s2[i..]);
        if x >= len {
            // we want the longest prefix that's closest to the end (probably? I think)
            len = x;
            off = i;
        }
        i += 1;
    }

    return (off, len);
}

#[cfg(test)]
mod test {
    use super::{length_of_common_prefix, longest_prefix_in, lzss_compress, lzss_compress_with_key, lzss_decompress_to_buffer};

    #[test]
    fn test_length_of_common_prefix() {
        // s1 is shorter
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3, 4], &[2, 3, 4, 5]), 0);
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3, 4], &[0, 3, 4, 5]), 1);
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3, 4], &[0, 1, 4, 5]), 2);
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3, 4], &[0, 1, 2, 5]), 3);
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3, 4], &[0, 1, 2, 3]), 4);

        // same length
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3], &[5, 1, 2, 3]), 0);
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3], &[0, 5, 2, 3]), 1);
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3], &[0, 1, 5, 3]), 2);
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3], &[0, 1, 2, 5]), 3);
        assert_eq!(length_of_common_prefix(&[0, 1, 2, 3], &[0, 1, 2, 3]), 4);

        // s2 is shorter
        assert_eq!(length_of_common_prefix(&[255, 254, 253, 252, 251], &[6, 6, 6]), 0);
        assert_eq!(length_of_common_prefix(&[255, 254, 253, 252, 251], &[255, 6, 9]), 1);
        assert_eq!(length_of_common_prefix(&[255, 254, 253, 252, 251], &[255, 254, 2]), 2);
        assert_eq!(length_of_common_prefix(&[255, 254, 253, 252, 251], &[255, 254, 253]), 3);
    }

    #[test]
    fn test_longest_prefix() {
        // trivial: no prefix
        assert_eq!(longest_prefix_in(&[], &[]).1, 0);
        assert_eq!(longest_prefix_in(&[], &[1, 2, 3, 4]).1, 0);
        assert_eq!(longest_prefix_in(&[1, 2, 3, 4, 5], &[]).1, 0);
        assert_eq!(longest_prefix_in(&[0, 1, 2, 3, 4], &[5, 6, 7, 8]).1, 0);
        assert_eq!(longest_prefix_in(&[0, 1, 2, 3, 4], &[1, 1, 1]).1, 0);

        assert_eq!(longest_prefix_in(&[0, 0, 0, 0, 0], &[1, 2, 3, 0, 0, 0, 4, 5, 6]), (3, 3));

        assert_eq!(longest_prefix_in(&[1, 2, 3, 4, 5], &[42, 255, 255, 255, 1, 7, 8, 9]), (4, 1));
        assert_eq!(longest_prefix_in(&[1, 2, 3, 4, 5], &[42, 255, 255, 255, 1, 2, 7, 8, 9]), (4, 2));
        assert_eq!(longest_prefix_in(&[1, 2, 3, 4, 5], &[42, 255, 255, 255, 1, 2, 3, 7, 8, 9]), (4, 3));
        assert_eq!(longest_prefix_in(&[1, 2, 3, 4, 5], &[42, 255, 255, 255, 1, 2, 3, 4, 7, 8, 9]), (4, 4));
        assert_eq!(longest_prefix_in(&[1, 2, 3, 4, 5], &[42, 255, 255, 255, 1, 2, 3, 4, 5, 7, 8, 9]), (4, 5));
        assert_eq!(longest_prefix_in(&[1, 2, 3, 4, 5], &[42, 255, 255, 255, 1, 2, 3, 4, 5, 6, 7, 8, 9]), (4, 5));
        assert_eq!(longest_prefix_in(&[1, 2, 3, 4, 5], &[42, 255, 255, 255, 1, 2, 3, 4, 5, 6, 7, 8, 9]), (4, 5));

        assert_eq!(longest_prefix_in(&[1, 2, 3, 1, 2, 3, 4, 1, 2, 3, 4, 5], &[2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5]), (7, 3));

        assert_eq!(longest_prefix_in(&[1, 2, 3, 1, 2, 3, 4, 1, 2, 3, 4, 5], &[0, 9, 1, 2, 3, 4, 5, 6, 7, 8]), (2, 3));
    }

    #[test]
    fn lzss_test_decompress_simple_1() {
        // uncompressed
        let compressed = [5, 0, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5];
        let mut decompressed = [0; 5];
        let expected = [1, 2, 3, 4, 5];
        lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
        assert_eq!(decompressed, expected);
    }

    #[test]
    fn lzss_test_decompress_simple_2() {
        // four uncompressed bytes, then repeat these four bytes
        let compressed = [8, 0, 0, 0, 19, 0, 0, 0, 255, 0, 0, 0, 1, 2, 3, 4, 255, 4, 4];
        let mut decompressed = [0; 8];
        let expected = [1, 2, 3, 4, 1, 2, 3, 4];
        lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
        assert_eq!(decompressed, expected);
    }

    #[test]
    fn lzss_test_decompress_simple_3() {
        // four uncompressed bytes, then repeat them, then repeat all eight again
        let compressed = [16, 0, 0, 0, 22, 0, 0, 0, 255, 0, 0, 0, 1, 2, 3, 4, 255, 4, 4, 255, 8, 8];
        let mut decompressed = [0; 16];
        let expected = [1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4];
        lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
        assert_eq!(decompressed, expected);
    }

    #[test]
    fn lzss_test_decompress_simple_4() {
        // compression key inserted twice into the stream
        let compressed = [2, 0, 0, 0, 16, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
        let mut decompressed = [0; 2];
        let expected = [255, 255];
        lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
        assert_eq!(decompressed, expected);
    }

    #[test]
    fn lzss_test_decompress_simple_5() {
        // compression key inserted twice, then both repeated once
        let compressed = [4, 0, 0, 0, 19, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 2, 2];
        let mut decompressed = [0; 4];
        let expected = [255, 255, 255, 255];
        lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
        assert_eq!(decompressed, expected);
    }

    #[test]
    fn lzss_test_compress_simple_1() {
        // bytes without any repeating pattern
        let raw = [1, 2, 3, 4, 5, 6, 7, 8];
        let compressed = lzss_compress(&raw).unwrap();
        let expected = [8, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(&compressed, &expected);
    }

    #[test]
    fn lzss_test_compress_simple_2() {
        // compression key is in the keystream
        let raw: std::vec::Vec<u8> = (0..256).map(|b| b.try_into().unwrap()).collect();
        let compressed = lzss_compress(&raw).unwrap();
        let mut expected = vec![0, 1, 0, 0, 13, 1, 0, 0, 0, 0, 0, 0];
        expected.push(0);
        expected.push(0);
        for i in 1..256 {
            expected.push(i.try_into().unwrap());
        }
        assert_eq!(&compressed, &expected);
    }

    #[test]
    fn lzss_test_compress_simple_3() {
        // compression key is in the keystream
        let raw: std::vec::Vec<u8> = (0..256).map(|b| b.try_into().unwrap()).collect();
        let compressed = lzss_compress(&raw).unwrap();
        let mut expected = vec![0, 1, 0, 0, 13, 1, 0, 0, 0, 0, 0, 0];
        expected.push(0);
        expected.push(0);
        for i in 1..256 {
            expected.push(i.try_into().unwrap());
        }
        assert_eq!(&compressed, &expected);
    }

    #[test]
    fn lzss_test_compress_simple_4() {
        // single repetition of the first 4 bytes; here the decompression key is > offset
        let raw = [1, 1, 1, 1, 2, 2, 2, 3, 4, 5, 6, 1, 1, 1, 1, 3, 3];
        let compressed = lzss_compress_with_key(&raw, 255).unwrap();
        let expected = vec![17, 0, 0, 0, 28, 0, 0, 0, 255, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 3, 4, 5, 6, 255, 11, 4, 3, 3];
        assert_eq!(&compressed, &expected);
        let mut decompressed = [0; 17];
        lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
        assert_eq!(&decompressed, &raw);
    }

    #[test]
    fn lzss_test_compress_simple_5() {
        // like lzss_test_compress_simple_4, but the compression key is < offset
        let raw = [1, 1, 1, 1, 2, 2, 2, 3, 4, 5, 6, 1, 1, 1, 1, 3, 3];
        let compressed = lzss_compress_with_key(&raw, 0).unwrap();
        let expected = vec![17, 0, 0, 0, 28, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 3, 4, 5, 6, 0, 12, 4, 3, 3];
        assert_eq!(&compressed, &expected);
        let mut decompressed = [0; 17];
        lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
        assert_eq!(&decompressed, &raw);
    }

    #[test]
    fn lzss_test_random_1() {
        // found through randomized property testing
        let raw = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 178, 103, 59, 3, 178, 103, 59, 3, 1, 2];
        let compressed = lzss_compress(&raw).unwrap();
        let expected = vec![27, 0, 0, 0, 32, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 5, 4, 4, 5, 9, 8, 4, 178, 103, 59, 3, 5, 4, 4, 1, 2];
        assert_eq!(&compressed, &expected);
        let mut decompressed = [0; 27];
        lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
        assert_eq!(&decompressed, &raw);
    }

    #[cfg(feature = "proptest")]
    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig{cases: 10_000_000, .. proptest::prelude::ProptestConfig::default() })]
        #[test]
        fn lzss_proptest_tiny(key in proptest::num::u8::ANY, data in proptest::collection::vec(proptest::num::u8::ANY, 0..100)) {
            let compressed = lzss_compress_with_key(&data, key).unwrap();
            let mut decompressed = vec![0; data.len()];
            lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
            assert_eq!(&decompressed, &data);
        }

        #[test]
        fn lzss_proptest_small(data in proptest::collection::vec(proptest::num::u8::ANY, 0..1_000)) {
            let compressed = lzss_compress(&data).unwrap();
            let mut decompressed = vec![0; data.len()];
            lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
            assert_eq!(&decompressed, &data);
        }
    }

    #[cfg(feature = "proptest")]
    proptest::proptest! {
        // if running these tests crashes your machine (on linux), use ulimit -v 10000000 to limit memory to 10GB

        #[test]
        fn lzss_proptest_medium(data in proptest::collection::vec(proptest::num::u8::ANY, 1_000..10_000)) {
            let compressed = lzss_compress(&data).unwrap();
            let mut decompressed = vec![0; data.len()];
            lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
            assert_eq!(&decompressed, &data);
        }

        #[test]
        fn lzss_proptest_big(data in proptest::collection::vec(proptest::num::u8::ANY, 10_000..100_000)) {
            let compressed = lzss_compress(&data).unwrap();
            let mut decompressed = vec![0; data.len()];
            lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
            assert_eq!(&decompressed, &data);
        }

        #[test]
        fn lzss_proptest_huge(data in proptest::collection::vec(proptest::num::u8::ANY, 10_000_000..100_000_000)) {
            let compressed = lzss_compress(&data).unwrap();
            let mut decompressed = vec![0; data.len()];
            lzss_decompress_to_buffer(&compressed, &mut decompressed).unwrap();
            assert_eq!(&decompressed, &data);
        }
    }

    #[test]
    fn lzss_test() {
        todo!("test with actual game data?")
    }

    #[test]
    fn lz4_test() {
        todo!("test with actual game data?")
    }

    #[test]
    fn zstd_test() {
        todo!("test with actual game data?")
    }
}
