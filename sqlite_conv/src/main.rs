extern crate rusqlite;
extern crate tocs;
extern crate walkdir;

extern crate sqliteio;
use sqliteio::SqliteIO as _;

use tocs::cs3::tbl as cs3_tbl;
use tocs::cs3::tbl::entry_data as cs3_entry_data;
use tocs::cs3::tbl::Entry as CS3Entry;

pub fn main() {
    let mut db = rusqlite::Connection::open("cs3tbl.db").unwrap();
    db.execute("pragma foreign_keys = on", []).unwrap();

    let tx = db.transaction().unwrap();

    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Generic>(&tx).unwrap();
    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::CharRevise>(&tx).unwrap();
    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::CompHelpData>(&tx).unwrap();
    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Item>(&tx).unwrap();
    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Magic>(&tx).unwrap();
    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::MagicBo>(&tx).unwrap();
    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::SlotCost>(&tx).unwrap();
    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Status>(&tx).unwrap();
    sqlite_recreate_table_destroy_if_exists_unsafe::<cs3_entry_data::Voice>(&tx).unwrap();

    let cwd = std::env::current_dir().unwrap();
    let base_path = cwd.join(std::path::Path::new("private-test-data/cs3/data/text/"));
    for (file_id, direntry) in walkdir::WalkDir::new(base_path).into_iter().enumerate() {
        let direntry = direntry.unwrap();
        let path = direntry.path();
        if direntry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            println!("{}", path.display());
/*
            let path_components: std::vec::Vec<_> = path.components().collect();
            let dbpath = (path_components.split_at(path_components.len() - 4).1)
                .iter()
                .fold(std::path::PathBuf::new(), |a, b| a.join(b.as_os_str()));
            tx.execute("insert into files (id, path) values (?1, ?2)", rusqlite::params![file_id, dbpath.display().to_string()])
                .unwrap();
*/
            // TODO: add paths
            // TODO: 'r#type' -> 'type' in derive(SqliteIO)
            let data = cs3_tbl::fix(&std::fs::read(path).unwrap());
            let tbl = cs3_tbl::load(&data).unwrap();
            for (n, tblentry) in tbl.entries.iter().enumerate() {
                match tblentry {
                    CS3Entry::Generic(data) => do_insert_unsafe(&tx, data).unwrap(),
                    CS3Entry::CharRevise(data) => do_insert_unsafe(&tx, data).unwrap(),
                    CS3Entry::CompHelpData(data) => do_insert_unsafe(&tx, data).unwrap(),
                    CS3Entry::Item(data) => do_insert_unsafe(&tx, data).unwrap(),
                    CS3Entry::Magic(data) => do_insert_unsafe(&tx, data).unwrap(),
                    CS3Entry::MagicBo(data) => do_insert_unsafe(&tx, data).unwrap(),
                    CS3Entry::SlotCost(data) => do_insert_unsafe(&tx, data).unwrap(),
                    CS3Entry::Status(data) => do_insert_unsafe(&tx, data).unwrap(),
                    CS3Entry::Voice(data) => do_insert_unsafe(&tx, data).unwrap(),
                }
            }
        }
    }

    tx.commit().unwrap()
}

// WARNING: The following functions could be extremely dangerous in a general settings:
// We are interpolating the 'table' string directly into the sql statement, without sanitization.
// This bears the risk of SQL injection.
// Since the table names all come from an attribute which we supply ourselves in the source code, this should not become a problem.
pub fn sqlite_schema_unsafe<T: sqliteio::SqliteIO>() -> String {
    let s = format!(
        "create table {} (\n{}\n)",
        T::sqlite_table(),
        T::sqlite_columns().iter().map(|c| format!("  '{c}'")).collect::<std::vec::Vec<_>>().join(",\n")
    );
    s
}

pub fn sqlite_recreate_table_destroy_if_exists_unsafe<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
    tx.execute(&format!("drop table if exists {}", T::sqlite_table()), [])?;
    tx.execute(&sqlite_schema_unsafe::<T>(), [])?;
    Ok(())
}

pub fn insert_unsafe(tx: &rusqlite::Transaction, table: &str, values: &[&dyn rusqlite::ToSql]) -> rusqlite::Result<usize> {
    let s = format!(
        "insert into {} values ({})",
        table,
        (0..values.len()).map(|i| format!("?{}", i + 1)).collect::<std::vec::Vec<_>>().join(", ")
    );
    let mut insert = tx.prepare_cached(&s).unwrap();
    insert.execute(values)
}

pub fn select_one_unsafe<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction) -> rusqlite::Result<T> {
    let s = format!("select * from {}", T::sqlite_table());
    tx.query_row(&s, [], |row| Ok(T::sqlite_row_read(row, 0)?.0))
}

pub fn do_insert_unsafe<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction, x: &T) -> rusqlite::Result<()> {
    insert_unsafe(&tx, T::sqlite_table(), &x.sqlite_to_tup())?;
    Ok(())
}
