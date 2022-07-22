extern crate clap;
extern crate rusqlite;
extern crate tocs;
extern crate walkdir;

use clap::Parser;
use std::io::Write as _;
use std::str::FromStr as _;

extern crate sqliteio;

use tocs::cs3::tbl as cs3_tbl;
use tocs::cs3::tbl::entry_data as cs3_entry_data;
use tocs::cs3::tbl::Entry as CS3Entry;
use tocs::cs3::tbl::TblEntry as _;

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
    },
    /// Read data from a sqlite database and generate corresponding tbl files
    To_Tbl {
        /// path of the database file from which data should be read
        #[clap(long)]
        db_path: std::path::PathBuf,
        /// path to the directory that the tbls should be written to
        #[clap(long)]
        tbl_path: std::path::PathBuf,
        /// overwrite any existing tbl files
        #[clap(short, long)]
        force: bool,
    },
}

pub fn main() {
    match Args::parse() {
        Args::To_Sql { tbl_path, db_path } => {
            to_new_db(&tbl_path, &db_path);
        }
        Args::To_Tbl { db_path, tbl_path, force } => {
            to_tbl(&db_path, &tbl_path, force);
        }
    }
}

fn to_new_db(tblpath: &std::path::Path, dbpath: &std::path::Path) {
    if dbpath.exists() {
        panic!("Database output path {} exists, refusing to overwrite", dbpath.display());
    }

    let mut db = rusqlite::Connection::open(dbpath).expect("opening new database file");
    db.execute("pragma foreign_keys = on", []).unwrap();

    let tx = db.transaction().unwrap();

    let create_tables = || -> rusqlite::Result<()> {
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Generic>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::ActiveVoiceTableData>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::AtBonus>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::CharRevise>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::CharData>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::CompHelpData>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::GameDifficulty>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Item>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::ItemHelpData>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Magic>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::MagicBo>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::MasterQuartzDummy>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::QSText>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::SE>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::SlotCost>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Status>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::StatusP>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::TextTableData>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Voice>(&tx)?;
        Ok(())
    };
    match create_tables() {
        Ok(()) => (),
        Err(e) => panic!("Error creating sql tables: {e}"),
    };

    for direntry in walkdir::WalkDir::new(tblpath).into_iter() {
        let direntry = match direntry {
            Ok(x) => x,
            Err(e) => panic!("Error walking directory: {e}"),
        };
        let path = direntry.path();
        if direntry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            println!("Reading {}", path.display());
            let path_for_db = path.strip_prefix(tblpath).expect("child path should have parent as prefix").display().to_string();
            let data = match std::fs::read(path).map(|data| cs3_tbl::fix(&data)) {
                Ok(x) => x,
                Err(e) => panic!("Couldn't read tbl file: {e}"),
            };
            let tbl = match cs3_tbl::load(&data) {
                Ok(x) => x,
                Err(e) => panic!("Couldn't parse tbl file: {}", e.to_msg(&data)),
            };
            for (n, tblentry) in tbl.entries.iter().enumerate() {
                let insert_result = match tblentry {
                    CS3Entry::Generic(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::ActiveVoiceTableData(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::AtBonus(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::CharRevise(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::CharData(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::CompHelpData(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::GameDifficulty(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Item(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::ItemHelpData(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Magic(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::MagicBo(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::MasterQuartzDummy(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::QSText(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::SE(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::SlotCost(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Status(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::StatusP(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::TextTableData(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Voice(data) => sqlite_insert_unsafe(&tx, &path_for_db, data),
                };
                match insert_result {
                    Ok(_) => (),
                    Err(e) => panic!("Couldn't insert entry {} of {} into table: {}", n + 1, tbl.entries.len(), e),
                };
            }
        }
    }

    match tx.commit() {
        Ok(_) => (),
        Err(e) => panic!("couldn't commit database transaction: {e}"),
    };
}

// WARNING: The following functions could be extremely dangerous in a general settings:
// We are interpolating the 'table' string directly into the sql statement, without sanitization.
// This bears the risk of SQL injection.
// Since the table names all come from an attribute which we supply ourselves in the source code, this should not become a problem.
fn sqlite_schema_unsafe<T: sqliteio::SqliteIO>() -> String {
    let s = format!(
        "create table {} (\nfile,\n{}\n)",
        T::sqlite_table(),
        T::sqlite_columns()
            .iter()
            .map(|(cname, ctype)| format!("  '{cname}' {ctype}"))
            .collect::<std::vec::Vec<_>>()
            .join(",\n")
    );
    s
}

fn sqlite_recreate_table_destroy_if_exists_unsafe<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
    tx.execute(&format!("drop table if exists {}", T::sqlite_table()), [])?;
    tx.execute(&sqlite_schema_unsafe::<T>(), [])?;
    Ok(())
}

fn sqlite_insert_unsafe<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction, filename: &str, x: &T) -> rusqlite::Result<usize> {
    let mut vals: std::vec::Vec<&dyn rusqlite::ToSql> = vec![&filename];
    let x_vals = x.sqlite_to_tup();
    for e in x_vals.iter() {
        vals.push(e.as_ref());
    }
    let s = format!(
        "insert into {} values ({})",
        T::sqlite_table(),
        (0..vals.len()).map(|i| format!("?{}", i + 1)).collect::<std::vec::Vec<_>>().join(", ")
    );
    let mut insert = tx.prepare_cached(&s)?;
    insert.execute(&*vals)
}

fn sqlite_read_table_unsafe_or_panic<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction, table: &str) -> std::vec::Vec<(String, T)> {
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

fn to_tbl(dbpath: &std::path::Path, output_path: &std::path::Path, force: bool) {
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
        println!("Reading table {table}");
        match table.as_str() {
            "generic" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Generic>(&tx, table) {
                    let e = CS3Entry::Generic(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "ActiveVoiceTableData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::ActiveVoiceTableData>(&tx, table) {
                    let e = CS3Entry::ActiveVoiceTableData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "AtBonus" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::AtBonus>(&tx, table) {
                    let e = CS3Entry::AtBonus(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "char_revise" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::CharRevise>(&tx, table) {
                    let e = CS3Entry::CharRevise(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "CharData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::CharData>(&tx, table) {
                    let e = CS3Entry::CharData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "CompHelpData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::CompHelpData>(&tx, table) {
                    let e = CS3Entry::CompHelpData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "game_difficulty" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::GameDifficulty>(&tx, table) {
                    let e = CS3Entry::GameDifficulty(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "item" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Item>(&tx, table) {
                    let e = CS3Entry::Item(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "magic" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Magic>(&tx, table) {
                    let e = CS3Entry::Magic(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "magicbo" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MagicBo>(&tx, table) {
                    let e = CS3Entry::MagicBo(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "MasterQuartzDummy" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MasterQuartzDummy>(&tx, table) {
                    let e = CS3Entry::MasterQuartzDummy(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "se" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::SE>(&tx, table) {
                    let e = CS3Entry::SE(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "SlotCost" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::SlotCost>(&tx, table) {
                    let e = CS3Entry::SlotCost(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "status" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Status>(&tx, table) {
                    let e = CS3Entry::Status(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "status_p" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::StatusP>(&tx, table) {
                    let e = CS3Entry::StatusP(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            "voice" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Voice>(&tx, table) {
                    let e = CS3Entry::Voice(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert(vec![e]);
                }
            }
            s => panic!("unexpected table {}. this is likely a version mismatch or a bug.", s),
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
        println!("Writing {}", filename.display());
        if let Some(parent) = filename.parent() {
            match std::fs::create_dir_all(parent) {
                Ok(()) => (),
                Err(e) => println!("Couldn't create parent path of tbl {}: {}", filename.display(), e),
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
