extern crate clap;
extern crate rusqlite;
extern crate tocs;
extern crate walkdir;

use clap::Parser;
use indexmap::IndexMap;
use smol_str::SmolStr;

use std::io::Write as _;
use std::str::FromStr as _;

extern crate sqliteio;

use tocs::cs1::tbl::TblEntry as _;

use tocs::cs3::tbl as cs3_tbl;
use tocs::cs3::tbl::entry_data as cs3_entry_data;
use tocs::cs3::tbl::Entry as CS3Entry;

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
    match Args::parse() {
        Args::To_Sql { tbl_path, db_path, game } => {
            to_new_db(&tbl_path, &db_path, game);
        }
        Args::To_Tbl { db_path, tbl_path, game, force } => {
            match game {
                Game::CS1 => todo!(),
                Game::CS2 => todo!(),
                Game::CS3 => to_tbl_cs3(&db_path, &tbl_path, force),
            };
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
        Game::CS1 => &*tocs::tbl::SCHEMAS_CS1_ATOMIC,
        Game::CS2 => &*tocs::tbl::SCHEMAS_CS2_ATOMIC,
        Game::CS3 => &*tocs::tbl::SCHEMAS_CS3_ATOMIC,
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
            let path_for_db = path.strip_prefix(tblpath.as_ref()).expect("child path should have parent as prefix").display().to_string();
            let data = std::fs::read(path).map_err(|e| format!("couldn't read tbl file: {e}")).unwrap();
            let mut logger = tocs::tbl::WarningLogger::new();
            let entries = match tbl_version {
                Game::CS1 => tocs::tbl::deserialize_tbl_v1(&mut std::io::Cursor::new(&tocs::cs1::tbl::fix(&data)), &mut logger, true, &schemas)
                    .map_err(|e| format!("couldn't parse tbl file: {e}"))
                    .unwrap(),
                Game::CS2 => {
                    tocs::tbl::deserialize_tbl_v2(&mut std::io::Cursor::new(&data), &mut logger, true, &schemas)
                        .map_err(|e| format!("couldn't parse tbl file: {e}"))
                        .unwrap()
                        .1
                }
                Game::CS3 => {
                    tocs::tbl::deserialize_tbl_v2(&mut std::io::Cursor::new(&tocs::cs3::tbl::fix(&data)), &mut logger, true, &schemas)
                        .map_err(|e| format!("couldn't parse tbl file: {e}"))
                        .unwrap()
                        .1
                }
            };
            for (n, (header, values)) in entries.iter().enumerate() {
                let insert_result = if let Some(_) = schemas.get(header) {
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
        println!("Reading table {table}");
        match table.as_str() {
            "generic" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Generic>(&tx, table) {
                    let e = CS3Entry::Generic(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "ActiveVoiceTableData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::ActiveVoiceTableData>(&tx, table) {
                    let e = CS3Entry::ActiveVoiceTableData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "AssetCache" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::AssetCache>(&tx, table) {
                    let e = CS3Entry::AssetCache(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "AtBonus" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::AtBonus>(&tx, table) {
                    let e = CS3Entry::AtBonus(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "AttachTableData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::AttachTableData>(&tx, table) {
                    let e = CS3Entry::AttachTableData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "bgm" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Bgm>(&tx, table) {
                    let e = CS3Entry::Bgm(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "char_revise" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::CharRevise>(&tx, table) {
                    let e = CS3Entry::CharRevise(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "CharData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::CharData>(&tx, table) {
                    let e = CS3Entry::CharData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "CompHelpData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::CompHelpData>(&tx, table) {
                    let e = CS3Entry::CompHelpData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "EventTableData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::EventTableData>(&tx, table) {
                    let e = CS3Entry::EventTableData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "FootStepData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::FootStepData>(&tx, table) {
                    let e = CS3Entry::FootStepData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "game_difficulty" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::GameDifficulty>(&tx, table) {
                    let e = CS3Entry::GameDifficulty(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "hkitugi_lst" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::HkitugiLst>(&tx, table) {
                    let e = CS3Entry::HkitugiLst(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "item" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Item>(&tx, table) {
                    let e = CS3Entry::Item(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "item_q" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::ItemQ>(&tx, table) {
                    let e = CS3Entry::ItemQ(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "ItemHelpData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::ItemHelpData>(&tx, table) {
                    let e = CS3Entry::ItemHelpData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "KisinPilotData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::KisinPilotData>(&tx, table) {
                    let e = CS3Entry::KisinPilotData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "LinkAbText" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::LinkAbText>(&tx, table) {
                    let e = CS3Entry::LinkAbText(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "magic" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Magic>(&tx, table) {
                    let e = CS3Entry::Magic(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "magicbo" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MagicBo>(&tx, table) {
                    let e = CS3Entry::MagicBo(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MapJumpData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MapJumpData>(&tx, table) {
                    let e = CS3Entry::MapJumpData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MarkerTableData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MarkerTableData>(&tx, table) {
                    let e = CS3Entry::MarkerTableData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MasterQuartzBase" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MasterQuartzBase>(&tx, table) {
                    let e = CS3Entry::MasterQuartzBase(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MasterQuartzData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MasterQuartzData>(&tx, table) {
                    let e = CS3Entry::MasterQuartzData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MasterQuartzDummy" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MasterQuartzDummy>(&tx, table) {
                    let e = CS3Entry::MasterQuartzDummy(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MasterQuartzMemo" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MasterQuartzMemo>(&tx, table) {
                    let e = CS3Entry::MasterQuartzMemo(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MasterQuartzStatus" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MasterQuartzStatus>(&tx, table) {
                    let e = CS3Entry::MasterQuartzStatus(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MG08Card" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MG08Card>(&tx, table) {
                    let e = CS3Entry::MG08Card(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MG08CharMsg" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MG08CharMsg>(&tx, table) {
                    let e = CS3Entry::MG08CharMsg(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MG08EnemyAi" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MG08EnemyAI>(&tx, table) {
                    let e = CS3Entry::MG08EnemyAI(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MG08PlyMsg" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MG08PlyMsg>(&tx, table) {
                    let e = CS3Entry::MG08PlyMsg(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MG08PresetDeck" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MG08PresetDeck>(&tx, table) {
                    let e = CS3Entry::MG08PresetDeck(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "MG08Text" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::MG08Text>(&tx, table) {
                    let e = CS3Entry::MG08Text(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "NameTableData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::NameTableData>(&tx, table) {
                    let e = CS3Entry::NameTableData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "NameVis" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::NameVis>(&tx, table) {
                    let e = CS3Entry::NameVis(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "NaviTextData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::NaviTextData>(&tx, table) {
                    let e = CS3Entry::NaviTextData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "PlaceTableData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::PlaceTableData>(&tx, table) {
                    let e = CS3Entry::PlaceTableData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "PopupMsgFaceCharData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::PopupMsgFaceCharData>(&tx, table) {
                    let e = CS3Entry::PopupMsgFaceCharData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSBook" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSBook>(&tx, table) {
                    let e = CS3Entry::QSBook(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSChapter" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSChapter>(&tx, table) {
                    let e = CS3Entry::QSChapter(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSChar" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSChar>(&tx, table) {
                    let e = CS3Entry::QSChar(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSCook" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSCook>(&tx, table) {
                    let e = CS3Entry::QSCook(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSFish" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSFish>(&tx, table) {
                    let e = CS3Entry::QSFish(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSHelp" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSHelp>(&tx, table) {
                    let e = CS3Entry::QSHelp(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSMons" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSMons>(&tx, table) {
                    let e = CS3Entry::QSMons(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSRank" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSRank>(&tx, table) {
                    let e = CS3Entry::QSRank(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSText" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSText>(&tx, table) {
                    let e = CS3Entry::QSText(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "QSTitle" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::QSTitle>(&tx, table) {
                    let e = CS3Entry::QSTitle(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "SchoolGrade" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::SchoolGrade>(&tx, table) {
                    let e = CS3Entry::SchoolGrade(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "se" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::SE>(&tx, table) {
                    let e = CS3Entry::SE(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "SlotCost" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::SlotCost>(&tx, table) {
                    let e = CS3Entry::SlotCost(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "status" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Status>(&tx, table) {
                    let e = CS3Entry::Status(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "status_p" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::StatusP>(&tx, table) {
                    let e = CS3Entry::StatusP(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "TacticalBonus" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::TacticalBonus>(&tx, table) {
                    let e = CS3Entry::TacticalBonus(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "text_lst" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::TextLst>(&tx, table) {
                    let e = CS3Entry::TextLst(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "TextTableData" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::TextTableData>(&tx, table) {
                    let e = CS3Entry::TextTableData(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
                }
            }
            "voice" => {
                for (s, e) in sqlite_read_table_unsafe_or_panic::<cs3_entry_data::Voice>(&tx, table) {
                    let e = CS3Entry::Voice(e);
                    all_entries.entry(s).and_modify(|entries| entries.push(e.clone())).or_insert_with(|| vec![e]);
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
