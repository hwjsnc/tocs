#![forbid(unsafe_code)]

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
        /// Game that the file belongs to (CS1-5)
        game: tocs::tbl::Version,
        /// .tbl path
        path: std::path::PathBuf,
        /// Don't attempt to fix game data with hardcoded values.
        /// Cannot be used together with the --lenient option
        #[arg(short, long, group = "parse options")]
        raw: bool,
        /// If strict parsing fails, try again with lax consistency checks
        /// Cannot be used together with the --raw option
        #[arg(long, group = "parse options")]
        lenient: bool,
    },
    /// Try to parse a specific pkg file
    ///
    /// For now, only prints the file list
    Pkg { path: std::path::PathBuf },
    /// Try to parse a specific pka file
    ///
    /// For now, only prints the file list
    Pka { path: std::path::PathBuf },
    /// Try to parse a dds file
    ///
    /// Prints metadata, attempts to parse BC7
    ParseDds { path: std::path::PathBuf },
    /// Convert a .dds with BC7-encoded texture data to r8g8b8a8
    DdsConvertBC7 { source: std::path::PathBuf, target: std::path::PathBuf },
    /// Serialize builtin tbl schemas to json
    SerializeBuiltinTblSchemas {
        /// Game version for which the schemas should be serialized
        version: tocs::tbl::Version,
    },
    /// Parse json as tbl schemas
    ParseTblSchemas {
        /// Path to the .json
        path: std::path::PathBuf,
    },
}

pub fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    match Subcommand::parse() {
        Subcommand::All { raw } => parse_all(!raw),
        Subcommand::Tbl { game, path, raw, lenient: true } => {
            assert!(!raw);
            read_tbl(&path, &game)
        }
        Subcommand::Tbl { game, path, raw, .. } => check_tbl_roundtrip(&path, &game, !raw),
        Subcommand::Pkg { path } => parse_pkg(&path),
        Subcommand::Pka { path } => parse_pka(&path),
        Subcommand::ParseDds { path } => parse_dds(&path),
        Subcommand::DdsConvertBC7 { source, target } => convert_bc7(&source, &target),
        Subcommand::SerializeBuiltinTblSchemas { version } => serialize_builtin_schemas(&version),
        Subcommand::ParseTblSchemas { path } => parse_tbl_schemas(&path),
    }
}

pub fn parse_all(fix_data: bool) {
    let cwd = std::env::current_dir().unwrap();

    for (rel_path, version) in [
        ("private-test-data/cs1/nisa-pc-1.6/data/text/dat_us", tocs::tbl::Version::CS1En),
        ("private-test-data/cs2/nisa-pc-1.4.1/data/text/dat_us", tocs::tbl::Version::CS2En),
        ("private-test-data/cs3/nisa-pc-1.0.5/data/text/", tocs::tbl::Version::CS3),
        ("private-test-data/cs4/nisa-switch-1.0.0/text/", tocs::tbl::Version::CS4),
        ("private-test-data/cs5/nisa-pc-1.0.2/text/", tocs::tbl::Version::CS5),
    ]
    .into_iter()
    {
        let base_path = cwd.join(rel_path);
        for entry in walkdir::WalkDir::new(base_path) {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
                log::info!("trying file {}", path.display());
                check_tbl_roundtrip(path, &version, fix_data);
            }
            // TODO: parse .pkgs
        }
    }
}

pub fn read_tbl(path: &std::path::Path, version: &tocs::tbl::Version) {
    match tocs::tbl::read_file_with_fixes(path, version) {
        tocs::tbl::ReadTblResult::Ok(_) => {
            log::info!("file read successfully");
        }
        tocs::tbl::ReadTblResult::KnownFixApplied(_) => {
            log::info!("file read successfully with known fix");
        }
        tocs::tbl::ReadTblResult::Warn { .. } => {
            log::warn!("file read successfully with warnings");
        }
        tocs::tbl::ReadTblResult::Err(e) => {
            log::error!("Couldn't read tbl: {}", e);
        }
    }
}

pub fn check_tbl_roundtrip(path: &std::path::Path, version: &tocs::tbl::Version, fix_data: bool) {
    let original = {
        let mut data = std::fs::read(path).unwrap();
        if fix_data {
            _ = tocs::tbl::fixes::fix_in_place(version, &mut data);
        }
        data
    };
    match tocs::tbl::read_once_new(&original, version) {
        Ok(tbl) => {
            let mut serialized: std::vec::Vec<u8> = vec![];
            match tocs::tbl::write(&mut std::io::Cursor::new(&mut serialized), &tbl) {
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
                let filename = tocs::pkg::FileName::try_from(filename).unwrap();

                //log::info!("{filename}: decompressed size = {decompressed_size}, compressed size = {compressed_size} (0x{compressed_size:x}), data offset = {data_offset} (0x{data_offset:x}), compression = {compression:?}");
                let raw_data = tocs::pkg::get_data_raw_from_file(&mut f, &entry).unwrap();
                let data = tocs::pkg::get_data_from_file(&mut f, &entry).unwrap();
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
        log::debug!("pkg: {}", pkg.name.as_padded_byte_str().to_string_lossy());
        log::info!("pkg: {}", tocs::pka::PkgName::try_from(pkg.name).unwrap());
        for file in pkg.files {
            log::debug!("file: name = {}, hash = '{}'", file.name.as_padded_byte_str().to_string_lossy(), hexhex::Hex::new(file.hash));
            log::info!("file: name = {}, hash = '{}'", tocs::pka::FileName::try_from(file.name).unwrap(), hexhex::Hex::new(file.hash));
        }
    }
    for hash in hashes {
        log::info!(
            "hash {}: offset = {} (0x{:x}), compressed size = {} (0x{}), decompressed size = {}, compression = {}",
            hexhex::Hex::new(hash.hash),
            hash.offset,
            hash.offset,
            hash.compressed_size,
            hash.compressed_size,
            hash.decompressed_size,
            hash.compression
        );
    }
}

use std::io::Write as _;

pub fn parse_dds(path: &std::path::Path) {
    let data = std::fs::read(path).unwrap();
    let tex = tocs::dds::load_texture(&data).unwrap();
    match tex.header {
        tocs::dds::Header::Base(h) => {
            print_dds_base_header(&h);
        }
        tocs::dds::Header::WithDxt10(base, ext) => {
            print_dds_base_header(&base);
            print_dds_dx10_ext_header(&ext);
            let mut block_counts: [u64; 9] = [0, 0, 0, 0, 0, 0, 0, 0, 0];
            for i in 0..(base.width * base.height) / 16 {
                let x: u8 = tex.raw_data[16 * i as usize];
                if x & 1 != 0 {
                    block_counts[0] += 1;
                } else if x & 2 != 0 {
                    block_counts[1] += 1;
                } else if x & 4 != 0 {
                    block_counts[2] += 1;
                } else if x & 8 != 0 {
                    block_counts[3] += 1;
                } else if x & 16 != 0 {
                    block_counts[4] += 1;
                } else if x & 32 != 0 {
                    block_counts[5] += 1;
                } else if x & 64 != 0 {
                    block_counts[6] += 1;
                } else if x & 128 != 0 {
                    block_counts[7] += 1;
                } else {
                    block_counts[8] += 1;
                }
            }
            log::info!("BC7 block stats:");
            for (i, n) in block_counts.iter().enumerate() {
                log::info!(" - blocks with mode {i}: {n}");
            }
        }
    }
}

fn print_dds_base_header(h: &tocs::dds::BaseHeader) {
    let tocs::dds::BaseHeader {
        size,
        flags,
        height,
        width,
        pitch_or_linear_size,
        depth,
        mip_map_count,
        reserved_1,
        dds_pixel_format,
        caps,
        reserved_2,
    } = h;
    log::info!("DDS Base header:");
    log::info!(" - size: {size}");
    log::info!(" - flags: {flags} (0x{flags:x}, 0b{flags:b})");
    log::info!(" - height: {height}");
    log::info!(" - width: {width}");
    log::info!(" - pitch_or_linear_size: {pitch_or_linear_size}");
    log::info!(" - depth: {depth}");
    log::info!(" - mip_map_count: {mip_map_count}");
    for (i, x) in reserved_1.iter().enumerate() {
        log::info!(" - reserved_1[{i}]: {x}");
    }
    {
        let tocs::dds::PixelFormat {
            size,
            flags,
            id,
            rgb_bit_count,
            r_bit_mask,
            g_bit_mask,
            b_bit_mask,
            a_bit_mask,
        } = dds_pixel_format;
        log::info!(" - pixel format:");
        log::info!("   - size: {size}");
        log::info!("   - flags: {flags} (0x{flags:x}, 0b{flags:b})");
        log::info!("   - id: {:?} ('{}')", id, String::from_utf8_lossy(id.as_slice()));
        log::info!("   - rgb bit count: {rgb_bit_count}");
        log::info!("   - r bitmask: {r_bit_mask} (0x{r_bit_mask:x}, 0b{r_bit_mask:b})");
        log::info!("   - g bitmask: {g_bit_mask} (0x{g_bit_mask:x}, 0b{g_bit_mask:b})");
        log::info!("   - b bitmask: {b_bit_mask} (0x{b_bit_mask:x}, 0b{b_bit_mask:b})");
        log::info!("   - a bitmask: {a_bit_mask} (0x{a_bit_mask:x}, 0b{a_bit_mask:b})");
    }
    for (i, x) in caps.iter().enumerate() {
        log::info!(" - caps[{i}]: {x}");
    }
    log::info!(" - reserved_2: {reserved_2}");
}

fn print_dds_dx10_ext_header(h: &tocs::dds::Dxt10ExtensionHeader) {
    let tocs::dds::Dxt10ExtensionHeader {
        dxgi_format,
        resource_dimension,
        misc_flag,
        array_size,
        reserved,
    } = h;
    log::info!("DDS DX10 extension header:");
    log::info!(" - dxgi format: {dxgi_format}");
    log::info!(" - resource_dimension: {resource_dimension}");
    log::info!(" - misc_flag: {misc_flag}");
    log::info!(" - array_size: {array_size} (0x{array_size:x})");
    log::info!(" - reserved: {reserved}");
}

fn convert_bc7(source: &std::path::Path, target: &std::path::Path) {
    let data = std::fs::read(source).unwrap();
    let tex = tocs::dds::load_texture(&data).unwrap();
    match tex.header {
        tocs::dds::Header::Base(_) => {
            log::error!("No extension header");
        }
        tocs::dds::Header::WithDxt10(base, ext) => {
            if ext.dxgi_format == tocs::dds::dxgi_format::BC7_UNORM || ext.dxgi_format == tocs::dds::dxgi_format::BC7_UNORM_SRGB {
                let colors = tocs::bc7::decode(tex.raw_data, base.width.try_into().unwrap(), base.height.try_into().unwrap()).unwrap();
                let mut f = std::fs::OpenOptions::new().write(true).create(true).open(target).unwrap();
                let mut buf = std::io::BufWriter::new(&mut f);
                tocs::dds::write_header(
                    &mut buf,
                    &tocs::dds::Header::Base(tocs::dds::BaseHeader {
                        size: 124,
                        flags: 0x100f, // no idea what these mean tbh
                        height: base.height,
                        width: base.width,
                        pitch_or_linear_size: 0,
                        depth: 0,
                        mip_map_count: 0,
                        reserved_1: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        dds_pixel_format: tocs::dds::PixelFormat {
                            size: 32,
                            flags: 65, // not sure what these mean either
                            id: [0, 0, 0, 0],
                            rgb_bit_count: 32,
                            r_bit_mask: 0x000000ff,
                            g_bit_mask: 0x0000ff00,
                            b_bit_mask: 0x00ff0000,
                            a_bit_mask: 0xff000000,
                        },
                        caps: [0, 0, 0, 0],
                        reserved_2: 0,
                    }),
                )
                .unwrap();
                for c in colors {
                    buf.write_all(&[c.r, c.g, c.b, c.a]).unwrap();
                }
            }
        }
    }
}

fn serialize_builtin_schemas(version: &tocs::tbl::Version) {
    let schemas: &tocs::tbl::Schemas = match version {
        tocs::tbl::Version::CS1En => &tocs::tbl::schemas::CS1,
        tocs::tbl::Version::CS2En => &tocs::tbl::schemas::CS2,
        tocs::tbl::Version::CS3 => &tocs::tbl::schemas::CS3,
        tocs::tbl::Version::CS4 => &tocs::tbl::schemas::CS4,
        tocs::tbl::Version::CS5 => &tocs::tbl::schemas::CS5,
        tocs::tbl::Version::V1(_) => unreachable!(),
        tocs::tbl::Version::V2(_) => unreachable!(),
    };
    println!("{}", serde_json::to_string(schemas).unwrap())
}

fn parse_tbl_schemas(path: &std::path::Path) {
    let data = std::fs::read(path).unwrap();
    let schemas: tocs::tbl::Schemas = serde_json::from_slice(&data).unwrap();
    let _atomic = tocs::tbl::get_nf1_for_all(&schemas).unwrap();
    log::info!("ok");
}
