#![forbid(unsafe_code)]

use clap::Parser;
use indexmap::IndexMap;

use tocs::tbl::Attribute;

#[allow(non_camel_case_types, clippy::enum_variant_names)]
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
        /// game (CS1/CS2/CS3/CS4/CS5)
        #[arg(long)]
        game: tocs::tbl::Version,
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
    /// Convert a folder of script files to a sqlite database (Reverie only)
    Scripts_to_Sql {
        /// path of the script folder
        #[arg(long)]
        script_path: std::path::PathBuf,
        /// path of the new database file
        #[arg(long)]
        db_path: std::path::PathBuf,
    },
}

pub fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    match Args::parse() {
        Args::Tbl_to_Sql { tbl_path, db_path, game } => {
            tbl_to_new_db(&tbl_path, &db_path, &game);
        }
        Args::Pka_to_Sql { pka_path, db_path } => {
            pka_metadata_to_new_db(&pka_path, &db_path);
        }
        Args::Scripts_to_Sql { script_path, db_path } => {
            scripts_to_new_db(&script_path, &db_path);
        }
    }
}

fn tbl_to_new_db<P: AsRef<std::path::Path>>(tblpath: P, dbpath: P, tbl_version: &tocs::tbl::Version) {
    if dbpath.as_ref().exists() {
        panic!("database output path {} exists, refusing to overwrite.", dbpath.as_ref().display());
    }

    let mut db = rusqlite::Connection::open(dbpath).expect("opening new database file");
    db.execute("pragma foreign_keys = on", []).unwrap();

    let tx = db.transaction().unwrap();

    let create_tables = || -> rusqlite::Result<()> {
        let generic_schema: IndexMap<Attribute, tocs::tbl::AtomicType> = [("header", tocs::tbl::AtomicType::CUtf8), ("data", tocs::tbl::AtomicType::X(0))]
            .map(|(s, t)| (s.into(), t))
            .into_iter()
            .collect();
        sqlite_recreate_table_destroy_if_exists_unsafe(&tx, "generic", &generic_schema)?;
        for (table, schema) in tbl_version.schemas().iter() {
            sqlite_recreate_table_destroy_if_exists_unsafe(&tx, table.as_str(), schema)?;
        }
        Ok(())
    };
    create_tables().unwrap();

    for direntry in walkdir::WalkDir::new(tblpath.as_ref()).into_iter() {
        let direntry = direntry.unwrap();
        let path = direntry.path();
        if direntry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
            log::info!("reading file {}", path.display());
            let path_for_db = path.strip_prefix(tblpath.as_ref()).expect("child path should have parent as prefix").display().to_string();
            let tbl = tocs::tbl::read_file_with_fixes(path, tbl_version).ignore_warnings().unwrap();
            for entry in tbl.get_entries().iter() {
                let insert_result = if tbl_version.schemas().contains_key(&entry.header) {
                    sqlite_insert_unsafe(&tx, entry.header.as_str(), &path_for_db, &entry.values)
                } else {
                    let mut values_with_header = IndexMap::new();
                    values_with_header.insert(Attribute::from("header"), tocs::tbl::AtomicValue::CUtf8(entry.header.as_str().into()));
                    for (attr, val) in entry.values.iter() {
                        values_with_header.insert(attr.clone(), val.clone());
                    }
                    sqlite_insert_unsafe(&tx, "generic", &path_for_db, &values_with_header)
                };
                insert_result.unwrap();
            }
        }
    }

    tx.commit().unwrap();
}

// WARNING: The following functions could be extremely dangerous in a general settings:
// We are interpolating the table and column names directly into the sql statement, without sanitization.
// This bears the risk of SQL injection.
// Since the table names all come from an attribute which we supply ourselves in the source code, this should not become a problem.

fn sqlite_recreate_table_destroy_if_exists_unsafe(tx: &rusqlite::Transaction, table_name: &str, schema: &IndexMap<Attribute, tocs::tbl::AtomicType>) -> rusqlite::Result<()> {
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

fn sqlite_insert_unsafe(tx: &rusqlite::Transaction, table: &str, filename: &str, values: &indexmap::IndexMap<Attribute, tocs::tbl::AtomicValue>) -> rusqlite::Result<usize> {
    let mut vals: std::vec::Vec<&dyn rusqlite::ToSql> = vec![&filename];
    for x in values.values() {
        vals.push(x);
    }
    let s = format!("insert into '{}' values ({})", table, std::iter::repeat("?").take(vals.len()).collect::<std::vec::Vec<_>>().join(", "),);
    let mut insert = tx.prepare_cached(&s)?;
    insert.execute(&*vals)
}

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

    tx.commit().unwrap();
}

fn scripts_to_new_db<P: AsRef<std::path::Path>>(base_path: P, dbpath: P) {
    if dbpath.as_ref().exists() {
        panic!("database output path {} exists, refusing to overwrite.", dbpath.as_ref().display());
    }

    let mut db = rusqlite::Connection::open(dbpath).expect("opening new database file");
    db.execute("pragma foreign_keys = on", []).unwrap();

    let tx = db.transaction().unwrap();

    tx.execute(
        "create table scripts (
    file_name text primary key not null,
    script_name text not null,
    name_padding int not null
) strict;",
        [],
    )
    .unwrap();

    tx.execute(
        "create table script_entries (
    file_name text not null,
    alignment int not null,
    entry_name text not null,
    foreign key(file_name) references scripts(file_name)
) strict;",
        [],
    )
    .unwrap();

    tx.execute(
        "create table ActionTable (
    file_name text not null,

    id int,
    category int,
    target_type int,
    target_range int,
    target_size int,

    unk4 real,
    unk5 real,
    unk6 real,

    cast_delay int,
    recovery_delay int,

    eff_1 int,
    param_1_1 int,
    param_1_2 int,
    param_1_3 int,
    eff_2 int,
    param_2_1 int,
    param_2_2 int,
    param_2_3 int,
    eff_3 int,
    param_3_1 int,
    param_3_2 int,
    param_3_3 int,
    eff_4 int,
    param_4_1 int,
    param_4_2 int,
    param_4_3 int,
    eff_5 int,
    param_5_1 int,
    param_5_2 int,
    param_5_3 int,

    unk10 int,

    unk18 int,
    flags text,
    animation text,
    name text,

    foreign key(file_name) references scripts(file_name)
) strict;",
        [],
    )
    .unwrap();

    tx.execute(
        "create table AlgoTable (
    file_name text not null,
    action_or_magic_id int,
    condition int,
    chance int,
    use_limit int,
    target_priority int,
    unk_byte_1 int,
    unk_byte_2 int,
    cond_param_1 int,
    cond_param_2 int,
    cond_param_3 int,
    target_param int,
    use_limit_after_retry int,
    unk6 int,
    foreign key(file_name) references scripts(file_name)
) strict;",
        [],
    )
    .unwrap();

    tx.execute(
        "create table SummonTable (
    file_name text not null,
    unk1 int,
    unk2 int,
    monster text,
    foreign key(file_name) references scripts(file_name)
) strict;",
        [],
    )
    .unwrap();
    tx.execute(
        "create table generic_entry (
    file_name text not null,
    name text not null,
    data blob not null,
    foreign key(file_name) references scripts(file_name)
) strict;",
        [],
    )
    .unwrap();

    for direntry in walkdir::WalkDir::new(base_path.as_ref()).into_iter() {
        let direntry = direntry.unwrap();
        let path = direntry.path();
        if direntry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("dat")) {
            println!("reading file {}", path.display());
            let path_for_db = path.strip_prefix(base_path.as_ref()).expect("child path should have parent as prefix").display().to_string();
            let data = std::fs::read(path).unwrap();
            let aldat = tocs::script::parse(data.as_slice()).unwrap();
            assert!(tocs::script::serialize(&aldat).unwrap() == data);
            tx.execute(
                "insert into scripts (file_name, script_name, name_padding) values (?, ?, ?)",
                [
                    &path_for_db as &dyn rusqlite::types::ToSql,
                    &aldat.name.as_str() as _,
                    match aldat.name_padding {
                        tocs::script::ScriptNamePadding::None => &0,
                        tocs::script::ScriptNamePadding::Once => &1,
                        tocs::script::ScriptNamePadding::Twice => &2,
                    },
                ],
            )
            .unwrap();
            for entry in aldat.entries.iter() {
                tx.execute(
                    "insert into script_entries (file_name, alignment, entry_name) values (?, ?, ?)",
                    [
                        &path_for_db as &dyn rusqlite::types::ToSql,
                        aldat.alignment.get(&entry.name().to_owned()).unwrap() as _,
                        &entry.name().to_str() as _,
                    ],
                )
                .unwrap();
                insert_script_entry_data(&tx, &path_for_db, entry);
            }
        }
    }

    tx.commit().unwrap();
}

fn insert_script_entry_data(tx: &rusqlite::Transaction, script_name: &str, entry: &tocs::script::ScriptEntry) {
    use tocs::script::{ActionTableEntry, AlgoTableEntry, ScriptEntry, SummonTableEntry};

    match entry {
        ScriptEntry::ActionTable(entries) => {
            for ActionTableEntry {
                id,
                category,
                target_type,
                target_range,
                target_size,
                unk4,
                unk5,
                unk6,
                cast_delay,
                recovery_delay,
                effects,
                unk10,
                unk18,
                flags,
                animation,
                name,
            } in entries
            {
                tx.execute(
                    "insert into ActionTable values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    &[
                        &script_name as &dyn rusqlite::types::ToSql,
                        &id as _,
                        &category as _,
                        &target_type as _,
                        &target_range as _,
                        &target_size as _,
                        &unk4 as _,
                        &unk5 as _,
                        &unk6 as _,
                        &cast_delay as _,
                        &recovery_delay as _,
                        &effects[0].id as _,
                        &effects[0].parameters[0] as _,
                        &effects[0].parameters[1] as _,
                        &effects[0].parameters[2] as _,
                        &effects[1].id as _,
                        &effects[1].parameters[0] as _,
                        &effects[1].parameters[1] as _,
                        &effects[1].parameters[2] as _,
                        &effects[2].id as _,
                        &effects[2].parameters[0] as _,
                        &effects[2].parameters[1] as _,
                        &effects[2].parameters[2] as _,
                        &effects[3].id as _,
                        &effects[3].parameters[0] as _,
                        &effects[3].parameters[1] as _,
                        &effects[3].parameters[2] as _,
                        &effects[4].id as _,
                        &effects[4].parameters[0] as _,
                        &effects[4].parameters[1] as _,
                        &effects[4].parameters[2] as _,
                        &unk10 as _,
                        &unk18 as _,
                        &flags.as_str() as _,
                        &animation.as_str() as _,
                        &name.as_str() as _,
                    ] as &[_],
                )
                .unwrap();
            }
        }
        ScriptEntry::AlgoTable(entries) => {
            for AlgoTableEntry {
                action_or_magic_id,
                condition,
                chance,
                use_limit,
                target_priority,
                unk_byte_1,
                unk_byte_2,
                cond_param_1,
                cond_param_2,
                cond_param_3,
                target_param,
                use_limit_after_retry,
                unk6,
            } in entries
            {
                tx.execute(
                    "insert into AlgoTable values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    &[
                        &script_name as &dyn rusqlite::types::ToSql,
                        &action_or_magic_id as _,
                        &condition as _,
                        &chance as _,
                        &use_limit as _,
                        &target_priority as _,
                        &unk_byte_1 as _,
                        &unk_byte_2 as _,
                        &cond_param_1 as _,
                        &cond_param_2 as _,
                        &cond_param_3 as _,
                        &target_param as _,
                        &use_limit_after_retry as _,
                        &unk6 as _,
                    ] as &[_],
                )
                .unwrap();
            }
        }
        ScriptEntry::SummonTable(entries) => {
            for SummonTableEntry { unk1, unk2, monster } in entries {
                tx.execute(
                    "insert into SummonTable values (?, ?, ?, ?)",
                    &[&script_name as &dyn rusqlite::types::ToSql, &unk1 as _, &unk2 as _, &monster.as_str() as _] as &[_],
                )
                .unwrap();
            }
        }
        ScriptEntry::Generic { name, data } => {
            tx.execute(
                "insert into generic_entry values (?, ?, ?)",
                [&script_name as &dyn rusqlite::types::ToSql, &name.as_str() as _, &data.as_slice() as _],
            )
            .unwrap();
        }
    }
}
