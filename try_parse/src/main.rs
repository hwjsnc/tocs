extern crate clap;
extern crate ioutil;
extern crate tocs;

use clap::Parser;

#[derive(Debug, Parser)]
enum Subcommand {
    /// Try to parse all data files
    ///
    /// Print the first error for each (if any)
    All {
        /// Don't attempt to fix game data with hardcoded values
        #[clap(short, long)]
        raw: bool,
    },
    /// Try to parse a specific file. Also print
    ///
    /// Print information about all the files
    File {
        game: GameVersion,
        path: std::path::PathBuf,
        /// Don't attempt to fix game data with hardcoded values
        #[clap(short, long)]
        raw: bool,
        /// Print data for parsed entries as text, to the extent possible
        #[clap(short, long)]
        print_data: bool,
    },
}

#[derive(Debug)]
enum GameVersion {
    CS1,
    CS2,
    CS3,
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
            _ => Err("invalid/unknown game"),
        }
    }
}

pub fn main() {
    match Subcommand::parse() {
        Subcommand::All { raw } => parse_all(!raw),
        Subcommand::File { game, path, raw, print_data } => match game {
            GameVersion::CS1 => parse_file_cs1(&path, !raw, print_data),
            GameVersion::CS2 => todo!(),
            GameVersion::CS3 => todo!(),
        },
    }
}

pub fn parse_all(fix_data: bool) {
    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs1/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            let original = if fix_data {
                tocs::cs1::tbl::fix(&std::fs::read(path).unwrap())
            } else {
                std::fs::read(path).unwrap()
            };
            match tocs::cs1::tbl::load(&original) {
                Ok(tbl) => {
                    match tocs::cs1::tbl::store(&tbl) {
                        Ok(serialized) if serialized == original => {
                            //println!("{}: ok!", path.display());
                        }
                        Ok(serialized) => {
                            for i in 0..std::cmp::max(serialized.len(), original.len()) {
                                if serialized.get(i) != original.get(i) {
                                    println!(
                                        "{}: roundtrip not identical to input, first difference at {}, expected {:?}, got {:?}",
                                        path.display(),
                                        i,
                                        original.get(i),
                                        serialized.get(i)
                                    );
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}: serialize failed: {:?}", path.display(), e)
                        }
                    }
                }
                Err(e) => {
                    println!("{}: Error: {}", path.display(), e.to_msg(&original));
                }
            }
        }
    }

    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs2/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            let original = std::fs::read(path).unwrap();
            match tocs::cs2::tbl::load(&original) {
                Ok(tbl) => match tocs::cs2::tbl::store(&tbl) {
                    Ok(serialized) if serialized == original => {
                        //println!("{}: ok!", path.display());
                    }
                    Ok(serialized) => {
                        for i in 0..std::cmp::max(serialized.len(), original.len()) {
                            if serialized.get(i) != original.get(i) {
                                println!(
                                    "{}: roundtrip not identical to input, first difference at {}, expected {:?}, got {:?}",
                                    path.display(),
                                    i,
                                    original.get(i),
                                    serialized.get(i)
                                );
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("{}: serialize failed: {:?}", path.display(), e)
                    }
                },
                Err(e) => {
                    println!("{}: Error: {}", path.display(), e.to_msg(&original));
                }
            }
        }
    }

    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs3/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            let original = if fix_data {
                tocs::cs3::tbl::fix(&std::fs::read(path).unwrap())
            } else {
                std::fs::read(path).unwrap()
            };
            match tocs::cs3::tbl::load(&original) {
                Ok(tbl) => match tocs::cs3::tbl::store(&tbl) {
                    Ok(serialized) if serialized == original => {
                        //println!("{}: ok!", path.display());
                    }
                    Ok(serialized) => {
                        for i in 0..std::cmp::max(serialized.len(), original.len()) {
                            if serialized.get(i) != original.get(i) {
                                println!(
                                    "{}: roundtrip not identical to input, first difference at {}, expected {:?}, got {:?}",
                                    path.display(),
                                    i,
                                    original.get(i),
                                    serialized.get(i)
                                );
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("{}: serialize failed: {:?}", path.display(), e)
                    }
                },
                Err(e) => {
                    println!("{}: Error: {}", path.display(), e.to_msg(&original));
                }
            }
        }
    }
}

pub fn parse_file_cs1(path: &std::path::Path, fix_data: bool, print_data: bool) {
    let data = if fix_data {
        tocs::cs1::tbl::fix(&std::fs::read(path).unwrap())
    } else {
        std::fs::read(path).unwrap()
    };
    let ctx = ioutil::ParseContext::start_of(&data);
    let (number_of_entries, mut ctx) = match ioutil::tbl::parse_tbl_header_v1(ctx) {
        Ok(result) => result,
        Err(e) => {
            panic!("Error: {}", e.to_msg(&data));
        }
    };
    println!("number of entries: {number_of_entries}");
    let mut i = 0;
    loop {
        let parsers = std::collections::HashMap::<_, _>::new();
        match ioutil::tbl::parse_entry::<tocs::cs1::tbl::Entry>(&parsers, ctx.clone(), i + 1, number_of_entries) {
            Ok((entry, entry_ctx)) => {
                match entry {
                    tocs::cs1::tbl::Entry::Generic(tocs::cs1::tbl::entry_data::Generic { header, data }) => {
                        println!(" - Generic entry ({} of {}, offset {} / 0x{:x})", i + 1, number_of_entries, ctx.position(), ctx.position());
                        println!("   - header name: '{}'", header);
                        println!("   - data length: {} / 0x{:x}", data.len(), data.len());
                        if print_data {
                            println!("   - data: {}", String::from_utf8_lossy(&data).escape_debug())
                        }
                    }
                }
                ctx = entry_ctx;
            }
            Err(e) => {
                if let ioutil::tbl::ParseErrorDetail::Primitive(ref e) = e.detail {
                    if matches!(e, ioutil::PrimitiveParseError::UnexpectedEOF { .. }) && e.position() == ctx.position() && i == number_of_entries {
                        println!("Reached EOF as expected, done.");
                        break;
                    }
                }
                panic!("Error: {}. {} bytes remaining.", e.to_msg(&data), data.len() - ctx.position());
            }
        };
        i += 1;
    }
}
