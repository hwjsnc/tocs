use smallvec::SmallVec;
use smol_str::SmolStr;

pub fn read_byte<F: std::io::Read>(f: &mut F) -> Result<u8, std::io::Error> {
    let a: [u8; 1] = read_bytes(f)?;
    Ok(a[0])
}

pub fn read_bytes<const N: usize, F: std::io::Read>(f: &mut F) -> Result<[u8; N], std::io::Error> {
    let mut buf: [u8; N] = [0; N];
    f.read_exact(&mut buf)?;
    Ok(buf)
}

pub enum ReadUtf8Error {
    IO(std::io::Error),
    Utf8(std::str::Utf8Error),
    UnexpectedPadding(u8),
}

pub fn read_cutf8_padded<F: std::io::Read>(f: &mut F, chunk_size: usize) -> Result<SmolStr, ReadUtf8Error> {
    let mut data: SmallVec<[u8; 64]> = SmallVec::new();
    let mut i = 0;
    loop {
        let x = read_byte(f).map_err(ReadUtf8Error::IO)?;
        i += 1;
        if x == 0 {
            break;
        } else {
            data.push(x);
        }
    }

    // read padding until we hit the chunk size
    while (i % chunk_size) != 0 {
        match read_byte(f) {
            Ok(0) => {}
            Ok(x) => {
                return Err(ReadUtf8Error::UnexpectedPadding(x));
            }
            Err(e) => {
                return Err(ReadUtf8Error::IO(e));
            }
        }
        i += 1;
    }

    // check utf-8
    std::str::from_utf8(&data).map(SmolStr::from).map_err(ReadUtf8Error::Utf8)
}
