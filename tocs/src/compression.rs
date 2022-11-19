use hashbrown::HashMap;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Compression {
    None,
    LZSS,
    LZ4,
    Zstd,
    Unknown(u32),
    // note: technically, Unknown(0) should equal None, Unknown(1) should equal LZSS, and so on
    // however, these unknown ones should never be constructed so having them be unequal is probably (hopefully) not a problem
}

impl std::fmt::Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Compression::None => write!(f, "uncompressed"),
            Compression::LZSS => write!(f, "LZSS"),
            Compression::LZ4 => write!(f, "LZ4"),
            Compression::Zstd => write!(f, "zstd"),
            Compression::Unknown(x) => write!(f, "unknown compression type (value {x} (0x{x:x}))"),
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
                let len: i32 = out.len().try_into().map_err(|_| "lz4 decompression error: output size too big".to_string())?;
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
                let level = 19; // This compression level approximately matches the one used in the switch port of CS4.
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

    if usize::try_from(decompressed_size).expect("32 bits please") != r#out.len() {
        return Err(format!(
            "lzss error: decompressed size mismatch ({} = 0x{:x} versus {} = 0x{:x})",
            decompressed_size,
            decompressed_size,
            r#out.len(),
            r#out.len()
        ));
    }
    if usize::try_from(compressed_size).expect("32 bits please") != r#in.len() {
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
                    next[0..len].copy_from_slice(&previous[copy_start..copy_start + len]);
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
    output.extend_from_slice(&u32::try_from(data.len()).map_err(|_| String::from("input size exceeds 32 bit maximum"))?.to_le_bytes()); // uncompressed size
    output.extend_from_slice(&[0, 0, 0, 0]); // compressed size, we fix this later
    output.extend_from_slice(&<u8 as Into<u32>>::into(key).to_le_bytes()); // compression key

    const PREFIX_LEN: usize = 2; // needs to be <= 4 for correctness
    const REMOVE_EMPTY: bool = false;

    #[allow(clippy::assertions_on_constants)]
    {
        assert!(PREFIX_LEN < 4 || REMOVE_EMPTY); // otherwise the hashmap may take up >4GB space
    }

    let mut offsets_to_try: HashMap<[u8; PREFIX_LEN], VecDeque<usize>> = HashMap::new();

    // must be called with the same offset and sequentially bigger i
    let update_offsets = |offsets: &mut HashMap<[u8; PREFIX_LEN], VecDeque<usize>>, i| {
        if i >= PREFIX_LEN {
            let new_off = i - PREFIX_LEN;
            let prefix: [u8; PREFIX_LEN] = data[new_off..new_off + PREFIX_LEN].try_into().unwrap();
            offsets
                .entry(prefix)
                .and_modify(|offsets| offsets.push_back(new_off))
                .or_insert_with(|| [new_off].into_iter().collect());
        }
        if i > 254 {
            let old_off = i - 255;
            let prefix: [u8; PREFIX_LEN] = data[old_off..old_off + PREFIX_LEN].try_into().unwrap();
            match offsets.entry(prefix) {
                hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    let offsets = entry.get_mut();
                    assert_eq!(offsets.pop_front(), Some(old_off));
                    if offsets.is_empty() && REMOVE_EMPTY {
                        entry.remove();
                    }
                }
                _ => unreachable!(),
            }
        }
    };

    let mut i = 0;
    while i < data.len() {
        // find longest prefix of the remaining input in the buffer of previous bytes
        let (previous, next) = data.split_at(i);
        update_offsets(&mut offsets_to_try, i);

        if i + PREFIX_LEN < data.len() {
            let prefix: [u8; PREFIX_LEN] = next[..PREFIX_LEN].try_into().unwrap();
            if let Some(offsets) = offsets_to_try.get(&prefix) {
                let mut len = 0;
                let mut off = 0;
                for o in offsets.iter().copied() {
                    if o + len >= i || len >= next.len() {
                        // taking the first longest occurence of the prefix is faster
                        break;
                    }
                    let x = length_of_common_prefix(&previous[o + PREFIX_LEN..], &data[i + PREFIX_LEN..]);
                    if x + PREFIX_LEN > len {
                        len = x + PREFIX_LEN;
                        off = o;
                    }
                }
                if len > 3 {
                    // inserting a reference takes 3 bytes in the compressed stream, so the length needs to be more than that if we want to save space.
                    off = previous.len() - off;
                    if off >= key.into() {
                        off += 1;
                    }
                    output.push(key);
                    output.push(off.try_into().unwrap());
                    output.push(len.try_into().unwrap());
                    for j in 1..len {
                        update_offsets(&mut offsets_to_try, i + j);
                    }
                    i += len;
                    continue;
                }
            }
        }
        if data[i] == key {
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
    std::cmp::min(s1.len(), s2.len())
}

#[cfg(test)]
mod test {
    use super::{length_of_common_prefix, lzss_compress, lzss_compress_with_key, lzss_decompress_to_buffer};

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

    #[test]
    fn lzss_test_random_2() {
        // found through randomized property testing
        let raw = [0, 118, 146, 96, 222, 118, 146, 96, 222];
        let compressed = lzss_compress_with_key(&raw, 0).unwrap();
        let expected = vec![9, 0, 0, 0, 21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 118, 146, 96, 222, 0, 5, 4];
        assert_eq!(&compressed, &expected);
        let mut decompressed = [0; 9];
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

    /*
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
    */
}
