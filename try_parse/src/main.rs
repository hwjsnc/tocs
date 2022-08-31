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
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    match Subcommand::parse() {
        Subcommand::All { raw } => parse_all(!raw),
        Subcommand::File { game, path, raw } => match game {
            GameVersion::CS1 => parse_file_cs1(&path, !raw),
            GameVersion::CS2 => parse_file_cs2(&path, !raw),
            GameVersion::CS3 => parse_file_cs3(&path, !raw),
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

pub fn parse_file_cs1(path: &std::path::Path, fix_data: bool) {
    let data = if fix_data {
        tocs::cs1::tbl::fix(&std::fs::read(path).unwrap())
    } else {
        std::fs::read(path).unwrap()
    };
    let mut log = tocs::tbl::WarningLogger::new();
    let _ = tocs::tbl::deserialize_tbl_v1(&mut std::io::Cursor::new(data), &mut log, true, &tocs::tbl::SCHEMAS_CS1_ATOMIC);
}

pub fn parse_file_cs2(path: &std::path::Path, _fix_data: bool) {
    let data = std::fs::read(path).unwrap();
    let mut log = tocs::tbl::WarningLogger::new();
    let _ = tocs::tbl::deserialize_tbl_v2(&mut std::io::Cursor::new(data), &mut log, true, &tocs::tbl::SCHEMAS_CS2_ATOMIC);
}

pub fn parse_file_cs3(path: &std::path::Path, fix_data: bool) {
    let data = if fix_data {
        tocs::cs1::tbl::fix(&std::fs::read(path).unwrap())
    } else {
        std::fs::read(path).unwrap()
    };
    let mut log = tocs::tbl::WarningLogger::new();
    let _ = tocs::tbl::deserialize_tbl_v2(&mut std::io::Cursor::new(data), &mut log, true, &tocs::tbl::SCHEMAS_CS3_ATOMIC);
}
