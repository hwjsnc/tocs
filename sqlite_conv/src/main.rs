extern crate rusqlite;
extern crate tocs;
extern crate walkdir;

extern crate sqliteio;

use tocs::cs3::tbl as cs3_tbl;
use tocs::cs3::tbl::entry_data as cs3_entry_data;
use tocs::cs3::tbl::Entry as CS3Entry;

pub fn main() {
    to_new_db(std::path::Path::new("../private-test-data/cs3/data/text/"), std::path::Path::new("cs3tbl.db"));
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
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::CharRevise>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::CompHelpData>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::GameDifficulty>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Item>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Magic>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::MagicBo>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::MasterQuartzDummy>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::SE>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::SlotCost>(&tx)?;
        sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Status>(&tx)?;
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
            println!("{}", path.display());
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
                    CS3Entry::Generic(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::ActiveVoiceTableData(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::CharRevise(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::CompHelpData(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::GameDifficulty(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Item(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Magic(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::MagicBo(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::MasterQuartzDummy(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::SE(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::SlotCost(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Status(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Voice(data) =>sqlite_insert_unsafe(&tx, &path_for_db, data),
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
        T::sqlite_columns().iter().map(|c| format!("  '{c}'")).collect::<std::vec::Vec<_>>().join(",\n")
    );
    s
}

fn sqlite_recreate_table_destroy_if_exists_unsafe<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
    tx.execute(&format!("drop table if exists {}", T::sqlite_table()), [])?;
    tx.execute(&sqlite_schema_unsafe::<T>(), [])?;
    Ok(())
}

fn sqlite_insert_unsafe<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction, filename: &str, x: &T) -> rusqlite::Result<usize> {
    let mut vals : std::vec::Vec::<&dyn rusqlite::ToSql> = vec![&filename];
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

fn sqlite_read_unsafe_or_panic<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction, filename: &str) -> std::vec::Vec<T> {
    let s = format!("select * from {} where file = ?", T::sqlite_table());
    let mut stmt = tx.prepare(&s).expect("statement");
    let rows = stmt.query_map([filename], |row| {
        let (e, next_column_index) = T::sqlite_row_read(row, 0)?;
        match row.get::<usize, rusqlite::types::Value>(next_column_index) {
            Ok(_) => panic!("Unexpected table schema (too many values)"),
            Err(_) => Ok(e)
        }
    }).expect("query");
    rows.into_iter().enumerate().map(|(n, row)| match row {
        Ok(x) => x,
        Err(e) => panic!("read error at row {}: {}", n+1, e)
    }).collect()
}

fn to_tbl() {
    // TODO:
    // open db
    // initialize empty hashmap (file -> tbl)
    // read table generic, for each entry:
    //   hashmap[file].entries.push(entry) (create if hashmap doesn't have entry)
    //   add entry header to file if it's not present yet
    // for all (other) tables in sqlite_schema:
    //   for each entry of the corresponding table:
    //     hashmap[file].entries.push(entry) (create if hashmap doesn't have entry)
    //   if any entry exists, add entry header to file
    // once all data has been read, add entry headers to tbl headers
    // serialize each tbl
    // write the tbls
    //
    // this won't give us bit-exact roundtrips (for example we drop headers without entries and we may reorder entries)
    // it remains to be seen if this leads to problems or not (i very much hope it doesn't).
}
