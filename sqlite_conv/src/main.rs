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

pub fn to_new_db(tblpath: &std::path::Path, dbpath: &std::path::Path) {
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
                    CS3Entry::Generic(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::ActiveVoiceTableData(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::CharRevise(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::CompHelpData(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::GameDifficulty(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Item(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Magic(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::MagicBo(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::MasterQuartzDummy(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::SE(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::SlotCost(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Status(data) => do_insert_unsafe(&tx, &path_for_db, data),
                    CS3Entry::Voice(data) => do_insert_unsafe(&tx, &path_for_db, data),
                };
                match insert_result {
                    Ok(()) => (),
                    Err(e) => panic!("Couldn't insert entry {} of {} into table: {}", n + 1, tbl.entries.len(), e),
                };
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
        "create table {} (\nfile,\n{}\n)",
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
    let mut insert = tx.prepare_cached(&s)?;
    insert.execute(values)
}

pub fn do_insert_unsafe<T: sqliteio::SqliteIO>(tx: &rusqlite::Transaction, filename: &str, x: &T) -> rusqlite::Result<()> {
    let mut vals : std::vec::Vec::<&dyn rusqlite::ToSql> = vec![&filename];
    let x_vals = x.sqlite_to_tup();
    for e in x_vals.iter() {
        vals.push(e.as_ref());
    }
    insert_unsafe(tx, T::sqlite_table(), &vals)?;
    Ok(())
}

pub fn to_tbl() {
    #[rustfmt::skip]
    let file_headers: std::collections::HashMap<&'static str, std::vec::Vec<&'static str>> = [
        ("t_active.tbl", vec!["ActiveVoiceTableData"]),
        ("t_assetcache.tbl", vec!["AssetCache"]),
        ("t_attach.tbl", vec!["AttachTableData"]),
        ("t_battle.tbl", vec!["BattleUIData"]),
        ("t_bgm.tbl", vec!["bgm"]),
        ("t_btlsys.tbl", vec!["TacticalBonus", "MissionText", "MissionData", "#struct OverRisePoint", "AtBonus", "BattleVoiceData_A", "BattleVoiceData_B", "BattleVoiceData_C", "BattleVoiceData_D", "BattleVoiceData_E", "BattleVoiceData_F", "BattleVoiceData_G", "BattleVoiceData_H", "BattleVoiceData_I"]),
        ("t_chrdata.tbl", vec!["CharData", "CheckEquipFlag", "KisinPilotData", "KisinPartnerData", "CharVoiceData"]),
        ("t_evtable.tbl", vec!["EventTableData"]),
        ("t_face.tbl", vec!["PopupMsgFaceCharData", "CutinCharData", "CutinCharDetailData"]),
        ("t_fish.tbl", vec!["fish_pnt", "rod_lst", "text_lst", "rod_base", "rod_parts", "fish_voice", "fish_chr"]),
        ("t_footstep.tbl", vec!["FootStepData"]),
        ("t_hikitugi.tbl", vec!["hkitugi_lst"]),
        ("t_horse.tbl", vec!["HorseData"]),
        ("t_item_en.tbl", vec!["item", "item_q"]),
        ("t_item_fr.tbl", vec!["item", "item_q"]),
        ("t_itemhelp.tbl", vec!["ItemHelpData", "CompHelpData"]),
        ("t_item.tbl", vec!["item", "item_q"]),
        ("t_jump.tbl", vec!["MapJumpData"]),
        ("t_likelv.tbl", vec!["LikeChara", "LikeLevelExp"]),
        ("t_linkab.tbl", vec!["LinkAbList", "LinkAbText", "LinkLevelExp", "LinkAbRateA", "LinkAbRateB"]),
        ("t_magic.tbl", vec!["magic", "magicbo", "btcalc"]),
        ("t_mapbgm.tbl", vec!["MapBgmTableData"]),
        ("t_marker.tbl", vec!["MarkerTableData"]),
        ("t_mg07.tbl", vec!["text_lst", "MG07PlayerList", "MG07ChrData"]),
        ("t_mg08.tbl", vec!["MG08Title", "MG08PlyMsg", "MG08CharMsg", "MG08PresetDeck", "MG08Card", "MG08Text", "MG08EnemyAi"]),
        ("t_mons.tbl", vec!["status", "char_revise"]),
        ("t_mstqrt.tbl", vec!["MasterQuartzStatus", "MasterQuartzBase", "MasterQuartzData", "MasterQuartzMemo", "MasterQuartzDummy", "#struct MasterQuartzDummy2"]),
        ("t_name.tbl", vec!["NameTableData"]),
        ("t_namevis.tbl", vec!["NameVis"]),
        ("t_naviicon.tbl", vec!["NaviIconTableData"]),
        ("t_navi.tbl", vec!["QSChapter", "NaviTextData"]),
        ("t_notebook.tbl", vec!["QSChapter", "QSBook"]),
        ("t_notecard.tbl", vec!["QSChapter", "QSCard"]),
        ("t_notechar.tbl", vec!["TopPageData", "QSChapter", "QSChar"]),
        ("t_notecook.tbl", vec!["QSCook", "QSCoolVoice"]),
        ("t_notefish.tbl", vec!["QSFish"]),
        ("t_notehelp.tbl", vec!["QSChapter", "QSHelp"]),
        ("t_notemons.tbl", vec!["QSChapter", "QSMons"]),
        ("t_notesynp.tbl", vec!["QSBookScp", "QSChapter", "QSBook"]),
        ("t_orb.tbl", vec!["BaseList", "OrbLineList"]),
        ("t_partysele.tbl", vec!["PTBadTerm"]),
        ("t_place.tbl", vec!["PlaceTableData"]),
        ("t_qu_cost.tbl", vec!["QuartzCost"]),
        ("t_quest.tbl", vec!["QSChapter", "QSPartResultAP", "QSTitle", "QSText", "QSRank", "SchoolGrade", "QSTrophyCheck"]),
        ("t_reverb.tbl", vec!["preset"]),
        ("t_se.tbl", vec!["se"]),
        ("t_shop.tbl", vec!["ShopGoods", "ShopTitle", "ShopConv", "ShopItem", "FPointItem"]),
        ("t_slot.tbl", vec!["SlotEp", "SlotCost"]),
        ("t_status.tbl", vec!["status_p", "status_revise", "char_revise", "game_difficulty", "field_attack_data"]),
        ("t_text.tbl", vec!["TextTableData"]),
        ("t_title.tbl", vec!["TitleData"]),
        ("t_trade.tbl", vec!["trade"]),
        ("t_vctiming_jp.tbl", vec!["VoiceTimingInfo", "VoiceTiming"]),
        ("t_vctiming.tbl", vec!["VoiceTimingInfo", "VoiceTiming"]),
        ("t_vctiming_us.tbl", vec!["VoiceTimingInfo", "VoiceTiming"]),
        ("t_voice.tbl", vec!["voice"]),
    ]
    .into_iter()
    .collect();
}
