use super::{
    deserialize_tbl_v1, deserialize_tbl_v2, read, schemas, serialize_tbl_v1, serialize_tbl_v2, write, AtomicSchemas, AtomicType, AtomicValue, Attribute, DeserializeError, Entry, Header,
    ReadTblResult, Tbl, Version, WarningLogger,
};

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
                for entry in tbl.get_entries() {
                    seen_headers.insert(entry.header.clone());
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
            let fixed = {
                let mut raw = std::fs::read(path).unwrap();
                _ = crate::tbl::fixes::fix_in_place(crate::tbl::Version::CS1En, &mut raw);
                raw
            };
            let mut logger = WarningLogger::<DeserializeError>::new();
            let entries = deserialize_tbl_v1::<_, /*STRICT =*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&fixed), &mut logger, &schemas::CS1_ATOMIC).unwrap();
            assert!(!logger.has_warnings());
            let serialized = {
                let mut serialized = std::io::Cursor::new(vec![]);
                let _ = serialize_tbl_v1(&mut serialized, &entries).unwrap();
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
                let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                let _ = serialize_tbl_v2(&mut serialized, &headers, &entries).unwrap();
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
            let fixed = {
                let mut raw = std::fs::read(path).unwrap();
                _ = crate::tbl::fixes::fix_in_place(crate::tbl::Version::CS3, &mut raw);
                raw
            };
            let mut logger = WarningLogger::<DeserializeError>::new();
            let (headers_with_counts, entries) = deserialize_tbl_v2::<_, /*STRICT =*/ true, /*LOG=*/ true>(&mut std::io::Cursor::new(&fixed), &mut logger, &schemas::CS3_ATOMIC).unwrap();
            assert!(!logger.has_warnings());
            let serialized = {
                let mut serialized = std::io::Cursor::new(vec![]);
                let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                let _ = serialize_tbl_v2(&mut serialized, &headers, &entries).unwrap();
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
                let headers: std::vec::Vec<_> = headers_with_counts.keys().map(|s| s.clone()).collect();
                let _ = serialize_tbl_v2(&mut serialized, &headers, &entries).unwrap();
                serialized.into_inner()
            };
            compare_data(&fixed, &serialized);
        }
    }
}

fn to_entry(schemas: &AtomicSchemas, (header, data): (&str, std::vec::Vec<(&str, &str)>)) -> Entry {
    let fallback_schema = [(Attribute::new("data"), AtomicType::X(0))].into_iter().collect();
    let header = Header::new(header).unwrap();
    let schema = schemas.get(&header).unwrap_or(&fallback_schema);
    let values = data
        .into_iter()
        .map(|(attr, s)| {
            let attr = Attribute::new(attr);
            let val = AtomicValue::from_str_nonstrict(schema.get(&attr).unwrap(), &s).unwrap();
            (attr, val)
        })
        .collect();
    Entry { header, values }
}

fn compare_entries(expected: &[Entry], actual: &[Entry]) {
    assert_eq!(actual.len(), expected.len());
    for row in 0..expected.len() {
        assert_eq!(actual[row].header, expected[row].header);
        assert_eq!(actual[row].values.len(), expected[row].values.len());
        for col in 0..actual[row].values.len() {
            let (actual_attr, actual_value) = actual[row].values.get_index(col).unwrap();
            let (expected_attr, expected_value) = expected[row].values.get_index(col).unwrap();
            assert_eq!(actual_attr, expected_attr);
            assert_eq!(actual_value.r#type(), expected_value.r#type());
            assert_eq!(actual_value.to_string(), expected_value.to_string());
        }
    }
}

#[test]
fn test_file_rw() {
    INIT_LOG.call_once(|| {
        simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).env().init().unwrap();
    });
    let schemas: AtomicSchemas = [
        ("foo", vec![("id", AtomicType::U16), ("abc", AtomicType::U16), ("data", AtomicType::X(2))]),
        ("bar", vec![("i16", AtomicType::I16), ("f32", AtomicType::F32), ("text", AtomicType::CUtf8), ("i8", AtomicType::I8)]),
    ]
    .into_iter()
    .map(|(hdr, schema)| (Header::new(&hdr).unwrap(), schema.into_iter().map(|(attr, t)| (Attribute::new(attr), t)).collect()))
    .collect();

    let original_rows: std::vec::Vec<Entry> = [
        ("foo", vec![("id", "12"), ("abc", "0"), ("data", "a0b0")]),
        ("foo", vec![("id", "13"), ("abc", "1"), ("data", "cccc")]),
        ("bar", vec![("i16", "-4"), ("f32", "0.428"), ("text", "aβが"), ("i8", "5")]),
        ("foo", vec![("id", "20"), ("abc", "2"), ("data", "1122")]),
        ("foobar", vec![("data", "0a0b0c0d")]),
        ("foobar", vec![("data", "99aabb")]),
        ("foo", vec![("id", "30"), ("abc", "5"), ("data", "0001")]),
        ("bar", vec![("i16", "19"), ("f32", "123.45678"), ("text", "lorem\nipsum"), ("i8", "-4")]),
        ("baz", vec![("data", "c0ffee")]),
    ]
    .into_iter()
    .map(|tup| to_entry(&schemas, tup))
    .collect();

    let tmp_dir = tempfile::tempdir().unwrap();
    let path = tmp_dir.path();
    //let path = tempfile::tempdir().unwrap().into_path();
    let tmp_file_1 = path.join(std::path::Path::new("v1.tbl"));
    let tmp_file_2 = path.join(std::path::Path::new("v2.tbl"));

    log::info!("file 1: {}", tmp_file_1.display());
    log::info!("file 2: {}", tmp_file_2.display());

    {
        log::info!("tbl version 1 write/read");
        crate::tbl::write_file(&tmp_file_1, &Tbl::new(Version::V1(&schemas), vec![], original_rows.clone())).unwrap();
        let read_rows = match crate::tbl::read_file(&tmp_file_1, Version::V1(&schemas)) {
            ReadTblResult::Ok(tbl) => tbl.into_entries(),
            ReadTblResult::Warn { warnings, .. } => {
                for warning in warnings {
                    log::error!("error reading tbl: {}", warning);
                }
                panic!();
            }
            ReadTblResult::Err(err) => {
                panic!("{err}");
            }
        };
        compare_entries(&read_rows, &original_rows);
    }

    {
        log::info!("tbl version 2 write/read");
        crate::tbl::write_file(
            &tmp_file_2,
            &Tbl::new(
                Version::V2(&schemas),
                ["foo", "foobar", "bar", "baz"].into_iter().map(|s| Header::new(s).unwrap()).collect::<std::vec::Vec<_>>(),
                original_rows.clone(),
            ),
        )
        .unwrap();
        let read_rows = match crate::tbl::read_file(&tmp_file_2, Version::V2(&schemas)) {
            ReadTblResult::Ok(tbl) => tbl.into_entries(),
            ReadTblResult::Warn { warnings, .. } => {
                for warning in warnings {
                    log::error!("error reading tbl: {}", warning);
                }
                panic!();
            }
            ReadTblResult::Err(err) => {
                panic!("{err}");
            }
        };
        compare_entries(&read_rows, &original_rows);
    }

    let original_rows: std::vec::Vec<Entry> = [
        ("foo", vec![("id", "7"), ("abc", "0"), ("data", "a0b0")]),
        ("foobar", vec![("data", "07429891")]),
        ("foo", vec![("id", "20"), ("abc", "2"), ("data", "1122")]),
        ("bar", vec![("i16", "-4"), ("f32", "0.428"), ("text", "ẞ"), ("i8", "-4")]),
        ("foobar", vec![("data", "ab")]),
        ("baz", vec![("data", "feed")]),
    ]
    .into_iter()
    .map(|tup| to_entry(&schemas, tup))
    .collect();

    // save the modified rows to the same files
    // the old data should be overwritten
    {
        log::info!("tbl version 1 write/read");
        crate::tbl::write_file(&tmp_file_1, &Tbl::new(Version::V1(&schemas), vec![], original_rows.clone())).unwrap();
        let read_rows = match crate::tbl::read_file(&tmp_file_1, Version::V1(&schemas)) {
            ReadTblResult::Ok(tbl) => tbl.into_entries(),
            ReadTblResult::Warn { warnings, .. } => {
                for warning in warnings {
                    log::error!("error reading tbl: {}", warning);
                }
                panic!();
            }
            ReadTblResult::Err(err) => {
                panic!("{err}");
            }
        };
        compare_entries(&read_rows, &original_rows);
    }

    {
        log::info!("tbl version 2 write/read");
        crate::tbl::write_file(
            &tmp_file_2,
            &Tbl::new(
                Version::V2(&schemas),
                ["foo", "foobar", "bar", "baz"].into_iter().map(|s| Header::new(s).unwrap()).collect::<std::vec::Vec<_>>(),
                original_rows.clone(),
            ),
        )
        .unwrap();
        let read_rows = match crate::tbl::read_file(&tmp_file_2, Version::V2(&schemas)) {
            ReadTblResult::Ok(tbl) => tbl.into_entries(),
            ReadTblResult::Warn { warnings, .. } => {
                for warning in warnings {
                    log::error!("error reading tbl: {}", warning);
                }
                panic!();
            }
            ReadTblResult::Err(err) => {
                panic!("{err}");
            }
        };
        compare_entries(&read_rows, &original_rows);
    }
}
