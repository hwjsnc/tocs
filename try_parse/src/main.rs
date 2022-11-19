extern crate clap;
extern crate tocs;

use clap::Parser;

#[derive(Debug, Parser)]
enum Subcommand {
    /// Try to parse all data files
    ///
    /// Print the first error for each (if any)
    All {
        /// Don't attempt to fix game data with hardcoded values
        #[arg(short, long)]
        raw: bool,
    },
    /// Try to parse a specific tbl file.
    ///
    /// Print information about all the files
    Tbl {
        game: GameVersion,
        path: std::path::PathBuf,
        /// Don't attempt to fix game data with hardcoded values
        #[arg(short, long)]
        raw: bool,
    },
    /// Try to parse a specific pkg file
    ///
    /// For now, only prints the file list
    Pkg { path: std::path::PathBuf },
    /// Try to parse a specific pka file
    ///
    /// For now, only prints the file list
    Pka { path: std::path::PathBuf },
}

// TODO: use generic parser in tocs?
#[derive(Clone, Copy, Debug)]
enum GameVersion {
    CS1,
    CS2,
    CS3,
    CS4,
    CS5,
}

impl std::str::FromStr for GameVersion {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, &'static str> {
        match s {
            "cs1" => Ok(GameVersion::CS1),
            "CS1" => Ok(GameVersion::CS1),
            "cs2" => Ok(GameVersion::CS2),
            "CS2" => Ok(GameVersion::CS2),
            "cs3" => Ok(GameVersion::CS3),
            "CS3" => Ok(GameVersion::CS3),
            "cs4" => Ok(GameVersion::CS4),
            "CS4" => Ok(GameVersion::CS4),
            "cs5" => Ok(GameVersion::CS5),
            "CS5" => Ok(GameVersion::CS5),
            "reverie" => Ok(GameVersion::CS5),
            "Reverie" => Ok(GameVersion::CS5),
            _ => Err("invalid/unknown game"),
        }
    }
}

pub fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    match Subcommand::parse() {
        Subcommand::All { raw } => parse_all(!raw),
        Subcommand::Tbl { game, path, raw } => match game {
            GameVersion::CS1 => {
                roundtrip_cs1(&path, !raw);
            }
            GameVersion::CS2 => {
                roundtrip_cs2(&path, !raw);
            }
            GameVersion::CS3 => {
                roundtrip_cs3(&path, !raw);
            }
            GameVersion::CS4 => {
                roundtrip_cs4(&path, !raw);
            }
            GameVersion::CS5 => {
                roundtrip_cs5(&path, !raw);
            }
        },
        Subcommand::Pkg { path } => parse_pkg(&path),
        Subcommand::Pka { path } => parse_pka(&path),
    }
}

pub fn parse_all(fix_data: bool) {
    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs1/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            log::info!("trying file {}", path.display());
            roundtrip_cs1(path, fix_data);
        }
        // TODO: parse .pkgs
    }

    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs2/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            log::info!("trying file {}", path.display());
            roundtrip_cs2(path, fix_data);
        }
    }

    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs3/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            log::info!("trying file {}", path.display());
            roundtrip_cs3(path, fix_data);
        }
    }

    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs4/nsw-1.0.0/text"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            log::info!("trying file {}", path.display());
            roundtrip_cs4(path, fix_data);
        }
    }

    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs5/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            log::info!("trying file {}", path.display());
            roundtrip_cs5(path, fix_data);
        }
    }
}

pub fn roundtrip_cs1(path: &std::path::Path, fix_data: bool) {
    let original = if fix_data {
        tocs::cs1::tbl::fix(&std::fs::read(path).unwrap())
    } else {
        std::fs::read(path).unwrap()
    };
    let mut log = tocs::tbl::WarningLogger::new();
    match tocs::tbl::deserialize_tbl_v1::<_, /*Strict=*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&original), &mut log, &tocs::tbl::schemas::CS1_ATOMIC) {
        Ok(entries) => {
            let mut serialized: std::vec::Vec<u8> = vec![];
            match tocs::tbl::serialize_tbl_v1(&mut std::io::Cursor::new(&mut serialized), &entries) {
                Ok(_written) => compare(&original, &serialized, path),
                Err(err) => log::error!("serialize failed: {err}"),
            }
        }
        Err(err) => {
            log::error!("deserialize failed: {err}");
        }
    }
}

pub fn roundtrip_cs2(path: &std::path::Path, _fix_data: bool) {
    let original = std::fs::read(path).unwrap();
    let mut log = tocs::tbl::WarningLogger::new();
    match tocs::tbl::deserialize_tbl_v2::<_, /*Strict=*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&original), &mut log, &tocs::tbl::schemas::CS2_ATOMIC) {
        Ok((header_counts, entries)) => {
            let mut serialized: std::vec::Vec<u8> = vec![];
            let headers: std::vec::Vec<_> = header_counts.keys().cloned().collect();
            match tocs::tbl::serialize_tbl_v2(&mut std::io::Cursor::new(&mut serialized), &headers, &entries) {
                Ok(_written) => compare(&original, &serialized, path),
                Err(err) => log::error!("serialize failed: {err}"),
            }
        }
        Err(err) => {
            log::error!("deserialize failed: {err}");
        }
    }
}

pub fn roundtrip_cs3(path: &std::path::Path, fix_data: bool) {
    let original = if fix_data {
        tocs::cs3::tbl::fix(&std::fs::read(path).unwrap())
    } else {
        std::fs::read(path).unwrap()
    };
    let mut log = tocs::tbl::WarningLogger::new();
    match tocs::tbl::deserialize_tbl_v2::<_, /*Strict=*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&original), &mut log, &tocs::tbl::schemas::CS3_ATOMIC) {
        Ok((header_counts, entries)) => {
            let mut serialized: std::vec::Vec<u8> = vec![];
            let headers: std::vec::Vec<_> = header_counts.keys().cloned().collect();
            match tocs::tbl::serialize_tbl_v2(&mut std::io::Cursor::new(&mut serialized), &headers, &entries) {
                Ok(_written) => compare(&original, &serialized, path),
                Err(err) => log::error!("serialize failed: {err}"),
            }
        }
        Err(err) => {
            log::error!("deserialize failed: {err}");
        }
    }
}

pub fn roundtrip_cs4(path: &std::path::Path, _fix_data: bool) {
    let original = std::fs::read(path).unwrap();
    let mut log = tocs::tbl::WarningLogger::new();
    match tocs::tbl::deserialize_tbl_v2::<_, /*STRICT=*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&original), &mut log, &tocs::tbl::schemas::CS4_ATOMIC) {
        Ok((header_counts, entries)) => {
            let mut serialized: std::vec::Vec<u8> = vec![];
            let headers: std::vec::Vec<_> = header_counts.keys().cloned().collect();
            match tocs::tbl::serialize_tbl_v2(&mut std::io::Cursor::new(&mut serialized), &headers, &entries) {
                Ok(_written) => compare(&original, &serialized, path),
                Err(err) => log::error!("serialize failed: {err}"),
            }
        }
        Err(err) => {
            log::error!("deserialize failed: {err}");
        }
    }
}

pub fn roundtrip_cs5(path: &std::path::Path, _fix_data: bool) {
    let original = std::fs::read(path).unwrap();
    let mut log = tocs::tbl::WarningLogger::new();
    match tocs::tbl::deserialize_tbl_v2::<_, /*STRICT=*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&original), &mut log, &tocs::tbl::schemas::CS5_ATOMIC) {
        Ok((header_counts, entries)) => {
            let mut serialized: std::vec::Vec<u8> = vec![];
            let headers: std::vec::Vec<_> = header_counts.keys().cloned().collect();
            match tocs::tbl::serialize_tbl_v2(&mut std::io::Cursor::new(&mut serialized), &headers, &entries) {
                Ok(_written) => compare(&original, &serialized, path),
                Err(err) => log::error!("serialize failed: {err}"),
            }
        }
        Err(err) => {
            log::error!("deserialize failed: {err}");
        }
    }
}

fn compare(original: &[u8], other: &[u8], path: &std::path::Path) {
    for i in 0..std::cmp::max(original.len(), other.len()) {
        if original.get(i) != other.get(i) {
            log::error!(
                "{}: roundtrip not identical to input, first difference at {}, expected {:?}, got {:?}",
                path.display(),
                i,
                original.get(i),
                other.get(i)
            );
            return;
        }
    }
}

pub fn parse_pkg(base_path: &std::path::Path) {
    for direntry in walkdir::WalkDir::new(base_path) {
        let direntry = direntry.unwrap();
        let path = direntry.path();
        if direntry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("pkg")) {
            let skip = [
                // files that seem broken
                // CS2:
                "C_NPC290_C00.pkg",
                "O_E6303.pkg",
            ];
            if skip.iter().map(std::ffi::OsStr::new).map(Some).any(|x| x == path.file_name()) {
                log::warn!("skipping file {}", path.display());
                continue;
            }
            //log::info!("trying file {}", path.display());
            let mut f = std::io::BufReader::new(std::fs::File::open(path).unwrap());
            let filelist = tocs::pkg::load_filelist(&mut f).unwrap();
            for entry in filelist {
                let tocs::pkg::FileListEntry {
                    filename,
                    decompressed_size: _,
                    compressed_size: _,
                    data_offset: _,
                    compression: _,
                } = &entry;

                //log::info!("{filename}: decompressed size = {decompressed_size}, compressed size = {compressed_size} (0x{compressed_size:x}), data offset = {data_offset} (0x{data_offset:x}), compression = {compression:?}");
                let raw_data = tocs::pkg::get_data_raw(&mut f, &entry).unwrap();
                let data = tocs::pkg::get_data(&mut f, &entry).unwrap();
                //std::io::stdout().write(&data);
                //let mut k = 0;
                let compressed = entry.compression.compress(&data).unwrap();
                if compressed != raw_data {
                    log::warn!(
                        "{filename}: compressed and uncompressed data does not match (recompressed data is {})",
                        match compressed.len().cmp(&raw_data.len()) {
                            std::cmp::Ordering::Greater => "bigger",
                            std::cmp::Ordering::Equal => "equally big",
                            std::cmp::Ordering::Less => "smaller",
                        }
                    );
                    /*
                    for i in 0..std::cmp::max(compressed.len(), raw_data.len()) {
                        let x = compressed.get(i);
                        let y = raw_data.get(i);
                        if x == y { println!("{i}: {x:?} == {y:?} ✓"); }
                        else { println!("{i}: {x:?} != {y:?} !!!"); k+=1; }
                        if k > 0 { k += 1 }
                        //if k > 10 { panic!(""); }
                    }
                    */
                } else {
                    log::info!("{filename}: ✓");
                }
                //let data2 = entry.compression.compress(&data).and_then(|compressed| entry.compression.decompress_with_exact_output_size(&compressed, entry.decompressed_size.try_into().unwrap())).map_err(|_| String::new());
                //if data2.as_ref() != Ok(&data) {
                //    log::error!("{filename}: decompress->compress->decompress is not the same as decompress!");
                //}
            }
        }
    }
}

pub fn parse_pka(path: &std::path::Path) {
    let mut f = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let (pkgs, hashes) = tocs::pka::load_filelist(&mut f).unwrap();
    for pkg in pkgs {
        log::info!("pkg: {}", pkg.name);
        for (name, hash) in pkg.filenames.iter().zip(pkg.hashes) {
            log::info!("file: name = {}, hash = {}", name, hex::encode(hash));
        }
    }
    for hash in hashes {
        log::info!(
            "hash {}: offset = {} (0x{:x}), compressed size = {} (0x{}), decompressed size = {}, compression = {}",
            hex::encode(hash.hash),
            hash.offset,
            hash.offset,
            hash.compressed_size,
            hash.compressed_size,
            hash.decompressed_size,
            hash.compression
        );
    }
}
