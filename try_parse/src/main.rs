extern crate tocs;

pub fn main() {
    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs1/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            let original = std::fs::read(path).unwrap();
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
            let original = std::fs::read(path).unwrap();
            let fixed = tocs::cs3::tbl::fix(&original);
            match tocs::cs3::tbl::load(&fixed) {
                Ok(tbl) => match tocs::cs3::tbl::store(&tbl) {
                    Ok(serialized) if serialized == fixed => {
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
