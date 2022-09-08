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
    To_Sql {
        /// path of the folder containing the tbls
        #[clap(long)]
        tbl_path: std::path::PathBuf,
        /// path of the new database file
        #[clap(long)]
        db_path: std::path::PathBuf,
        /// game (CS1/CS2/CS3)
        #[clap(long)]
        game: Game,
    },
    /// Read data from a sqlite database and generate corresponding tbl files
    To_Tbl {
        /// path of the database file from which data should be read
        #[clap(long)]
        db_path: std::path::PathBuf,
        /// path to the directory that the tbls should be written to
        #[clap(long)]
        tbl_path: std::path::PathBuf,
        /// game (CS1/CS2/CS3)
        #[clap(long)]
        game: Game,
        /// overwrite any existing tbl files
        #[clap(short, long)]
        force: bool,
    },
}

#[derive(Debug)]
enum Game {
    CS1,
    CS2,
    CS3,
}

impl std::str::FromStr for Game {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, &'static str> {
        match s {
            "cs1" => Ok(Game::CS1),
            "CS1" => Ok(Game::CS1),
            "cs2" => Ok(Game::CS2),
            "CS2" => Ok(Game::CS2),
            "cs3" => Ok(Game::CS3),
            "CS3" => Ok(Game::CS3),
            _ => Err("invalid/unknown game"),
        }
    }
}

pub fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    match Args::parse() {
        Args::To_Sql { tbl_path, db_path, game } => {
            to_new_db(&tbl_path, &db_path, game);
        }
        Args::To_Tbl { .. } => {
            todo!()
        }
    }
}

fn to_new_db<P: AsRef<std::path::Path>>(tblpath: P, dbpath: P, tbl_version: Game) {
    if dbpath.as_ref().exists() {
        panic!("database output path {} exists, refusing to overwrite.", dbpath.as_ref().display());
    }

    let mut db = rusqlite::Connection::open(dbpath).expect("opening new database file");
    db.execute("pragma foreign_keys = on", []).unwrap();

    let schemas = match tbl_version {
        Game::CS1 => &*tocs::tbl::schemas::CS1_ATOMIC,
        Game::CS2 => &*tocs::tbl::schemas::CS2_ATOMIC,
        Game::CS3 => &*tocs::tbl::schemas::CS3_ATOMIC,
    };

    let tx = db.transaction().unwrap();

    let create_tables = || -> rusqlite::Result<()> {
        let generic_schema: IndexMap<SmolStr, tocs::tbl::AtomicType> = [("header", tocs::tbl::AtomicType::CUtf8), ("data", tocs::tbl::AtomicType::X(0))]
            .map(|(s, t)| (SmolStr::from(s), t))
            .into_iter()
            .collect();
        sqlite_recreate_table_destroy_if_exists_unsafe(&tx, "generic", &generic_schema)?;
        for (table, schema) in schemas.iter() {
            sqlite_recreate_table_destroy_if_exists_unsafe(&tx, table, schema)?;
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
            let data = std::fs::read(path).map_err(|e| format!("couldn't read tbl file: {e}")).unwrap();
            let mut logger = tocs::tbl::WarningLogger::new();
            let entries = match tbl_version {
                Game::CS1 => tocs::tbl::deserialize_tbl_v1(&mut std::io::Cursor::new(&tocs::cs1::tbl::fix(&data)), &mut logger, true, schemas)
                    .map_err(|e| format!("couldn't parse tbl file: {e}"))
                    .unwrap(),
                Game::CS2 => {
                    tocs::tbl::deserialize_tbl_v2(&mut std::io::Cursor::new(&data), &mut logger, true, schemas)
                        .map_err(|e| format!("couldn't parse tbl file: {e}"))
                        .unwrap()
                        .1
                }
                Game::CS3 => {
                    tocs::tbl::deserialize_tbl_v2(&mut std::io::Cursor::new(&tocs::cs3::tbl::fix(&data)), &mut logger, true, schemas)
                        .map_err(|e| format!("couldn't parse tbl file: {e}"))
                        .unwrap()
                        .1
                }
            };
            for (n, (header, values)) in entries.iter().enumerate() {
                let insert_result = if schemas.contains_key(header) {
                    sqlite_insert_unsafe(&tx, header.as_ref(), &path_for_db, values)
                } else {
                    let mut values_with_header = IndexMap::new();
                    values_with_header.insert(SmolStr::from("header"), tocs::tbl::AtomicValue::CUtf8(header.clone()));
                    for (attr, val) in values {
                        values_with_header.insert(attr.clone(), val.clone());
                    }
                    sqlite_insert_unsafe(&tx, "generic", &path_for_db, &values_with_header)
                };
                insert_result.map_err(|e| format!("couldn't insert entry {} of {} into table: {}", n + 1, entries.len(), e)).unwrap();
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
