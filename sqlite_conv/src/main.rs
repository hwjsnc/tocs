extern crate clap;
extern crate rusqlite;
extern crate tocs;
extern crate walkdir;

use clap::Parser;
use indexmap::IndexMap;
use smol_str::SmolStr;

#[allow(non_camel_case_types)]
#[derive(Debug, Parser)]
enum Args {
    /// Convert a folder of tbl files to a sqlite database
    Tbl_to_Sql {
        /// path of the folder containing the tbls
        #[arg(long)]
        tbl_path: std::path::PathBuf,
        /// path of the new database file
        #[arg(long)]
        db_path: std::path::PathBuf,
        /// game (CS1/CS2/CS3)
        #[arg(long)]
        game: tocs::tbl::Version<'static>,
    },
    /// Read data from a sqlite database and generate corresponding tbl files
    To_Tbl {
        /// path of the database file from which data should be read
        #[arg(long)]
        db_path: std::path::PathBuf,
        /// path to the directory that the tbls should be written to
        #[arg(long)]
        tbl_path: std::path::PathBuf,
        /// game (CS1/CS2/CS3/CS4)
        #[arg(long)]
        game: tocs::tbl::Version<'static>,
        /// overwrite any existing tbl files
        #[arg(short, long)]
        force: bool,
    },
    /// Read .pka file list and convert to sqlite for analysis
    ///
    /// Content is not included in the database, only metadata
    Pka_to_Sql {
        /// path to the pka file
        #[arg(long)]
        pka_path: std::path::PathBuf,
        /// path of the new database file
        #[arg(long)]
        db_path: std::path::PathBuf,
    },
}

pub fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    match Args::parse() {
        Args::Tbl_to_Sql { tbl_path, db_path, game } => {
            tbl_to_new_db(&tbl_path, &db_path, game);
        }
        Args::To_Tbl { .. } => {
            todo!()
        }
        Args::Pka_to_Sql { pka_path, db_path } => {
            pka_metadata_to_new_db(&pka_path, &db_path);
        }
    }
}

fn tbl_to_new_db<P: AsRef<std::path::Path>>(tblpath: P, dbpath: P, tbl_version: tocs::tbl::Version) {
    if dbpath.as_ref().exists() {
        panic!("database output path {} exists, refusing to overwrite.", dbpath.as_ref().display());
    }

    let mut db = rusqlite::Connection::open(dbpath).expect("opening new database file");
    db.execute("pragma foreign_keys = on", []).unwrap();

    let tx = db.transaction().unwrap();

    let create_tables = || -> rusqlite::Result<()> {
        let generic_schema: IndexMap<SmolStr, tocs::tbl::AtomicType> = [("header", tocs::tbl::AtomicType::CUtf8), ("data", tocs::tbl::AtomicType::X(0))]
            .map(|(s, t)| (SmolStr::from(s), t))
            .into_iter()
            .collect();
        sqlite_recreate_table_destroy_if_exists_unsafe(&tx, "generic", &generic_schema)?;
        for (table, schema) in tbl_version.schemas().iter() {
            sqlite_recreate_table_destroy_if_exists_unsafe(&tx, table.as_str(), schema)?;
        }
        Ok(())
    };
    create_tables().map_err(|e| format!("couldn't create tables: {e}")).unwrap();

    for direntry in walkdir::WalkDir::new(tblpath.as_ref()).into_iter() {
        let direntry = direntry.map_err(|e| format!("error walking directory: {e}")).unwrap();
        let path = direntry.path();
        if direntry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            log::info!("reading file {}", path.display());
            let path_for_db = path.strip_prefix(tblpath.as_ref()).expect("child path should have parent as prefix").display().to_string();
            let tbl = tocs::tbl::read_file(path, tbl_version)
                .ignore_warnings()
                .map_err(|e| format!("couldn't parse tbl file {}: {}", path.display(), e))
                .unwrap();
            for (n, entry) in tbl.get_entries().iter().enumerate() {
                let insert_result = if tbl_version.schemas().contains_key(&entry.header) {
                    sqlite_insert_unsafe(&tx, entry.header.as_str(), &path_for_db, &entry.values)
                } else {
                    let mut values_with_header = IndexMap::new();
                    values_with_header.insert(SmolStr::from("header"), tocs::tbl::AtomicValue::CUtf8(entry.header.as_str().into()));
                    for (attr, val) in entry.values.iter() {
                        values_with_header.insert(attr.clone(), val.clone());
                    }
                    sqlite_insert_unsafe(&tx, "generic", &path_for_db, &values_with_header)
                };
                insert_result
                    .map_err(|e| format!("couldn't insert entry {} of {} into table: {}", n + 1, tbl_version.schemas().len(), e))
                    .unwrap();
            }
        }
    }

    tx.commit().map_err(|e| format!("couldn't commit database transaction: {e}")).unwrap();
}

// WARNING: The following functions could be extremely dangerous in a general settings:
// We are interpolating the table and column names directly into the sql statement, without sanitization.
// This bears the risk of SQL injection.
// Since the table names all come from an attribute which we supply ourselves in the source code, this should not become a problem.

fn sqlite_recreate_table_destroy_if_exists_unsafe(tx: &rusqlite::Transaction, table_name: &str, schema: &IndexMap<SmolStr, tocs::tbl::AtomicType>) -> rusqlite::Result<()> {
    tx.execute(&format!("drop table if exists '{}'", table_name), [])?;
    tx.execute(
        &format!(
            "create table '{}' (\nfile,\n{}\n)",
            table_name,
            schema.iter().map(|(cname, ty)| format!("  '{}' {}", cname, ty.sqlite_type())).collect::<std::vec::Vec<_>>().join(",\n")
        ),
        [],
    )?;
    Ok(())
}

fn sqlite_insert_unsafe(tx: &rusqlite::Transaction, table: &str, filename: &str, values: &indexmap::IndexMap<SmolStr, tocs::tbl::AtomicValue>) -> rusqlite::Result<usize> {
    let mut vals: std::vec::Vec<&dyn rusqlite::ToSql> = vec![&filename];
    for x in values.values() {
        vals.push(x);
    }
    let s = format!("insert into '{}' values ({})", table, std::iter::repeat("?").take(vals.len()).collect::<std::vec::Vec<_>>().join(", "),);
    let mut insert = tx.prepare_cached(&s)?;
    insert.execute(&*vals)
}

/*
TODO
fn sqlite_read_table_unsafe_or_panic(tx: &rusqlite::Transaction, table: &str) -> std::vec::Vec<(String, T)> {
    assert_eq!(table, T::sqlite_table());
    let mut columns = vec![("file".to_string(), "text")];
    columns.extend(T::sqlite_columns());
    let s = format!(
        "select\n{}\nfrom {}",
        columns.iter().map(|(cname, _ctype)| format!("  [{cname}]")).collect::<std::vec::Vec<String>>().join(",\n"),
        T::sqlite_table()
    );
    let mut stmt = match tx.prepare(&s) {
        Ok(x) => x,
        Err(e) => panic!("couldn't prepare select statement (incorrect schema / version?): {e}"),
    };
    let rows = stmt
        .query_map([], |row| {
            let file = row.get::<usize, String>(0)?;
            let (entry, next_column_index) = T::sqlite_row_read(row, 1)?;
            row.get::<usize, rusqlite::types::Value>(next_column_index).expect_err("select statement ensures matching schema");
            Ok((file, entry))
        })
        .expect("query");
    rows.enumerate()
        .map(|(n, row)| match row {
            Ok(x) => x,
            Err(e) => panic!("Error reading {}'th row: {}", n + 1, e),
        })
        .collect()
}

fn to_tbl_cs3(dbpath: &std::path::Path, output_path: &std::path::Path, force: bool) {
    // load data from db
    let mut db = match rusqlite::Connection::open(dbpath) {
        Ok(x) => x,
        Err(e) => panic!("couldn't open db file: {e}"),
    };
    db.execute("pragma foreign_keys = on", []).unwrap();
    let tx = db.transaction().unwrap();
    let mut all_entries = std::collections::HashMap::<String, std::vec::Vec<CS3Entry>>::new();
    let mut stmt = tx.prepare("select name from sqlite_schema").expect("sqlite_schema query");
    let tables: std::vec::Vec<String> = stmt
        .query_map([], |row| row.get::<usize, String>(0))
        .expect("sqlite_schema contents")
        .map(|row| row.expect("sqlite_schema row"))
        .collect();
    for table in tables.iter() {
        log::info!("Reading table {table}");
        match table.as_str() {
        }
    }
    drop(stmt);
    drop(tx);
    drop(db);

    // serialize data
    let mut serialized = std::collections::HashMap::<std::path::PathBuf, std::vec::Vec<u8>>::new();
    for (filename, entries) in all_entries.into_iter() {
        let headers = entries
            .iter()
            .map(|e| e.header_name())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<std::vec::Vec<_>>();
        let tbl = cs3_tbl::Tbl { headers, entries };
        let data = match cs3_tbl::store(&tbl) {
            Ok(x) => x,
            Err(e) => panic!("couldn't generate tbl {}: {:?}", filename, e), // TODO: non-debug error message
        };
        let relpath = std::path::PathBuf::from_str(&filename).expect("infallible");
        if relpath.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            panic!("file {} has a parent component. bailing out to avoid accidently clobbering random files.", filename);
        };
        let path = output_path.join(relpath);
        serialized.insert(path, data);
    }

    // write the tbls
    for (filename, data) in serialized.into_iter() {
        log::info!("Writing {}", filename.display());
        if let Some(parent) = filename.parent() {
            match std::fs::create_dir_all(parent) {
                Ok(()) => (),
                Err(e) => panic!("Couldn't create parent path of tbl {}: {}", filename.display(), e),
            };
        }
        let mut options = std::fs::OpenOptions::new();
        options.write(true);
        if force {
            options.create(true).truncate(true)
        } else {
            options.create_new(true)
        };
        let mut file = match options.open(&filename) {
            Ok(f) => f,
            Err(e) => panic!("couldn't open file {}: {}", filename.display(), e),
        };
        match file.write_all(&data) {
            Ok(()) => (),
            Err(e) => panic!("Couldn't write data to {}: {}", filename.display(), e),
        };
    }

    // note: we're not getting bit-exact roundtrips because we drop headers without entries and we may reorder entries
    // it remains to be seen if this leads to problems or not (i very much hope it doesn't).
}
*/

fn pka_metadata_to_new_db<P: AsRef<std::path::Path>>(pkapath: P, dbpath: P) {
    if dbpath.as_ref().exists() {
        panic!("database output path {} exists, refusing to overwrite.", dbpath.as_ref().display());
    }

    let mut db = rusqlite::Connection::open(dbpath).expect("opening new database file");
    //db.execute("pragma foreign_keys = on", []).unwrap();

    let tx = db.transaction().unwrap();

    tx.execute("drop table if exists files", []).unwrap();
    tx.execute("drop table if exists hashes", []).unwrap();
    tx.execute("create table files (pkg text, file text, hash blob) strict;", []).unwrap();
    tx.execute(
        "create table data (hash blob, offset integer, 'compressed size' integer, 'decompressed size' integer, compression text) strict;",
        [],
    )
    .unwrap();

    tx.execute("create index files_pkg on files (pkg);", []).unwrap();
    tx.execute("create index files_name on files (file);", []).unwrap();
    tx.execute("create index data_hash on data (hash);", []).unwrap();

    let mut f = std::io::BufReader::new(std::fs::File::open(pkapath).unwrap());
    let (pkgs, hashes) = tocs::pka::load_filelist(&mut f).unwrap();
    for pkg in pkgs {
        let p = tocs::pka::PkgName::try_from(pkg.name).unwrap();
        for entry in pkg.files {
            // special case for cs4:
            let filename;
            let f: &str;
            if entry.name.as_bytes_excluding_terminator_and_padding().is_empty() {
                f = "";
            } else {
                filename = tocs::pka::FileName::try_from(entry.name).unwrap();
                f = filename.as_str();
            }

            tx.execute("insert into files (pkg, file, hash) values (?1, ?2, ?3)", rusqlite::params![p.as_str(), f, entry.hash])
                .unwrap();
        }
    }
    for hash in hashes {
        tx.execute(
            "insert into data (hash, offset, 'compressed size', 'decompressed size', compression) values (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![hash.hash, hash.offset, hash.compressed_size, hash.decompressed_size, hash.compression.to_string()],
        )
        .unwrap();
    }

    tx.commit().map_err(|e| format!("couldn't commit database transaction: {e}")).unwrap();
}
