use super::{deserialize_tbl_v1, deserialize_tbl_v2, read, schemas, serialize_tbl_v1, serialize_tbl_v2, tbl_entries_legacy_convert, write, DeserializeError, ReadTblResult, Version, WarningLogger};

static INIT_LOG: std::sync::Once = std::sync::Once::new();

const TBL_TEST_FOLDERS: [(
    &str,
    Version,
    bool, // should we expect flawless loading & bit-identical round-trips for this?
    bool, // is the dataset comprehensive, i.e. should all headers in the schema appear in the dataset?
); 6] = [
    ("private-test-data/cs1/nisa-pc-1.6/data/text/dat_us/", Version::CS1En, false, true),
    ("private-test-data/cs2/nisa-pc-1.4.1/data/text/dat_us/", Version::CS2En, false, true),
    ("private-test-data/cs3/nisa-pc-1.0.5/data/text/", Version::CS3, false, true),
    ("private-test-data/cs3/softbrilliant-mod-1.0.6/data/text/", Version::CS3, false, false),
    ("private-test-data/cs4/nisa-switch-1.0.0/text/", Version::CS4, true, true),
    ("private-test-data/cs4/softbrilliant-mod-1.0.7/data/text/", Version::CS4, false, false),
];

fn compare_data(expected: &[u8], actual: &[u8]) {
    for i in 0..std::cmp::max(expected.len(), actual.len()) {
        let x1 = expected.get(i);
        let x2 = actual.get(i);
        if x1 != x2 {
            panic!("results differ, first difference at {i} (0x{i:x}). expected {x1:?}, found {x2:?}");
        }
    }
}

#[test]
fn test_read_all() {
    INIT_LOG.call_once(|| {
        simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();
    });
    let cwd = std::env::current_dir().unwrap();
    for (s, version, strict, comprehensive) in TBL_TEST_FOLDERS.iter() {
        let schema_headers: std::collections::BTreeSet<_> = version.schemas().keys().cloned().collect();
        let mut seen_headers = std::collections::BTreeSet::new();
        let base_path = cwd.to_path_buf().join(std::path::Path::new(s));
        for direntry in walkdir::WalkDir::new(base_path) {
            let direntry = direntry.unwrap();
            let path = direntry.path();
            if direntry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
                log::info!("READING FILE {}", path.display());
                let original = std::fs::read(path).unwrap();
                let tbl = match read(&mut std::io::Cursor::new(&original), *version) {
                    ReadTblResult::Ok(tbl) => tbl,
                    ReadTblResult::Warn { tbl, .. } => {
                        if *strict {
                            panic!("There shouldn't be any warnings, but there are!");
                        } else {
                            let mut serialized = std::io::Cursor::new(vec![]);
                            let _ = write(&mut serialized, &tbl).unwrap();
                        }
                        tbl
                    }
                    ReadTblResult::Err(e) => {
                        log::error!("Error!");
                        panic!("{}", e);
                    }
                };
                // serialize and compare
                let mut serialized = std::io::Cursor::new(vec![]);
                let _ = write(&mut serialized, &tbl).unwrap();
                let serialized = serialized.into_inner();
                if *strict {
                    compare_data(&original, &serialized);
                }
                // mark all entry headers as seen
                for (hdr, _) in tbl.get_entries() {
                    seen_headers.insert(hdr.clone());
                }
            }
        }
        if *comprehensive {
            let difference: std::collections::BTreeSet<_> = schema_headers.difference(&seen_headers).collect();
            if !difference.is_empty() {
                for hdr in difference {
                    log::error!("header '{hdr}' appears in the schemas but not in the dataset");
                }
                panic!("found unused headers (typo?)");
            }
        }
    }
}

#[test]
fn test_roundtrip_cs1_strict() {
    INIT_LOG.call_once(|| {
        simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
    });
    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs1/nisa-pc-1.6/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(parent) = path.parent() {
            if parent.file_name() == Some(&std::ffi::OsStr::new("dat")) {
                continue; // TODO: SJIS support for jp files would be nice
            }
        }
        if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
            log::info!("{}", path.display());
            let fixed = crate::cs1::tbl::fix(&std::fs::read(path).unwrap());
            let mut logger = WarningLogger::<DeserializeError>::new();
            let entries = deserialize_tbl_v1::<_, /*STRICT =*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&fixed), &mut logger, &schemas::CS1_ATOMIC).unwrap();
            assert!(!logger.has_warnings());
            let serialized = {
                let mut serialized = std::io::Cursor::new(vec![]);
                let e: &[_] = &entries;
                let _ = serialize_tbl_v1(&mut serialized, &tbl_entries_legacy_convert(e)).unwrap();
                serialized.into_inner()
            };
            compare_data(&fixed, &serialized);
        }
    }
}

#[test]
fn test_roundtrip_cs2_strict() {
    INIT_LOG.call_once(|| {
        simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
    });
    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs2/nisa-pc-1.4.1/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(parent) = path.parent() {
            if parent.file_name() == Some(&std::ffi::OsStr::new("dat")) {
                continue; // TODO: SJIS support for jp files would be nice
            }
        }
        if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
            log::info!("{}", path.display());
            let fixed = std::fs::read(path).unwrap();
            let mut logger = WarningLogger::<DeserializeError>::new();
            let (headers_with_counts, entries) = deserialize_tbl_v2::<_, /*STRICT =*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&fixed), &mut logger, &schemas::CS2_ATOMIC).unwrap();
            assert!(!logger.has_warnings());
            let serialized = {
                let mut serialized = std::io::Cursor::new(vec![]);
                let e: &[_] = &entries;
                let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                let _ = serialize_tbl_v2(&mut serialized, &headers, &tbl_entries_legacy_convert(&e)).unwrap();
                serialized.into_inner()
            };
            compare_data(&fixed, &serialized);
        }
    }
}

#[test]
fn test_roundtrip_cs3_strict() {
    INIT_LOG.call_once(|| {
        simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
    });
    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs3/nisa-pc-1.0.5/data/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
            log::info!("{}", path.display());
            let fixed = crate::cs3::tbl::fix(&std::fs::read(path).unwrap());
            let mut logger = WarningLogger::<DeserializeError>::new();
            let (headers_with_counts, entries) = deserialize_tbl_v2::<_, /*STRICT =*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&fixed), &mut logger, &schemas::CS3_ATOMIC).unwrap();
            assert!(!logger.has_warnings());
            let serialized = {
                let mut serialized = std::io::Cursor::new(vec![]);
                let e: &[_] = &entries;
                let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                let _ = serialize_tbl_v2(&mut serialized, &headers, &tbl_entries_legacy_convert(&e)).unwrap();
                serialized.into_inner()
            };
            compare_data(&fixed, &serialized);
        }
    }
}

#[test]
fn test_roundtrip_cs4_strict() {
    INIT_LOG.call_once(|| {
        simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
    });
    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.to_path_buf().join(std::path::Path::new("private-test-data/cs4/nisa-switch-1.0.0/text/"));
    for entry in walkdir::WalkDir::new(base_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().is_file() && path.extension() == Some(&std::ffi::OsStr::new("tbl")) {
            log::info!("{}", path.display());
            let fixed = std::fs::read(path).unwrap();
            let mut logger = WarningLogger::<DeserializeError>::new();
            let (headers_with_counts, entries) = deserialize_tbl_v2::<_, /*STRICT =*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&fixed), &mut logger, &schemas::CS4_ATOMIC).unwrap();
            assert!(!logger.has_warnings());
            let serialized = {
                let mut serialized = std::io::Cursor::new(vec![]);
                let e: &[_] = &entries;
                let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                let _ = serialize_tbl_v2(&mut serialized, &headers, &tbl_entries_legacy_convert(&e)).unwrap();
                serialized.into_inner()
            };
            compare_data(&fixed, &serialized);
        }
    }
}
