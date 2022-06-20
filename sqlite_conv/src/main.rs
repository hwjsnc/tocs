extern crate rusqlite;
extern crate tocs;
extern crate walkdir;

use tocs::cs3::tbl as cs3_tbl;
use tocs::cs3::tbl::entry_data as cs3_entry_data;
use tocs::cs3::tbl::Entry as CS3Entry;

pub fn main() {
    let mut db = rusqlite::Connection::open("cs3tbl.db").unwrap();

    db.execute("pragma foreign_keys = on", []).unwrap();

    let tx = db.transaction().unwrap();
    // create tables & indices
    tx.execute("drop table if exists tbl", []).unwrap();
    tx.execute("drop table if exists files", []).unwrap();
    tx.execute("create table files (id integer primary key, path)", []).unwrap();
    tx.execute("create unique index file_index on files (path);", []).unwrap();
    tx.execute(
        "create table tbl (
    file integer,
    [entry number] integer,
    attribute text,
    value any,
    primary key (file, [entry number], attribute),
    foreign key (file) references files(id)
)",
        [],
    )
    .unwrap();
    tx.execute("create index tbl_attr_value on tbl (attribute, value);", []).unwrap();
    tx.execute("create index tbl_value on tbl (value);", []).unwrap();

    // create views
    tx.execute("drop view if exists headers", []).unwrap();
    tx.execute(
        "create view headers as
select distinct value as 'header'
from tbl
where attribute = 'header name'",
        [],
    )
    .unwrap();
    tx.execute("drop view if exists generic_entries", []).unwrap();
    tx.execute(
        "create view generic_entries as
select
    tbl1.file as 'file',
    tbl1.[entry number] as 'entry number',
    (select value from tbl where file = tbl1.file and [entry number] = tbl1.[entry number] and attribute = 'header name') as 'header name',
    (select value from tbl where file = tbl1.file and [entry number] = tbl1.[entry number] and attribute = 'data') as 'data'
from tbl as tbl1
group by tbl1.file, tbl1.[entry number]
having count(*) = 2",
        [],
    )
    .unwrap();

    tx.execute("drop view if exists char_revise;", []).unwrap();
    tx.execute(
        "create view char_revise as
select
    files.path as file,
    entry.[entry number] as 'entry number',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'char') as char,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'hp') as hp,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'str') as str,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'def') as def,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'ats') as ats,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'adf') as adf,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'spd') as spd,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'brk') as brk
from tbl as entry
join files on files.id = entry.file
where entry.attribute = 'header name'
and entry.value = 'char_revise'",
        [],
    )
    .unwrap();

    tx.execute("drop view if exists CompHelpData", []).unwrap();
    tx.execute(
        "create view CompHelpData as
select
    entry.file as 'file',
    entry.[entry number] as 'entry number',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'type') as 'type',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'text') as 'text',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'value') as 'value'
from tbl as entry
where entry.attribute = 'header name'
and value = 'CompHelpData'",
        [],
    )
    .unwrap();

    tx.execute("drop view if exists item", []).unwrap();
    tx.execute(
        "create view item as
select
    files.path as 'file',
    entry.[entry number] as 'entry number',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'id') as 'id',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'character restriction') as 'character restriction',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'flags') as 'flags',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'type') as 'type',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'subtype') as 'subtype',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'element') as 'element',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'slash') as 'slash',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'thrust') as 'thrust',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'pierce') as 'pierce',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'strike') as 'strike',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'unknown byte') as 'unknown byte',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'target type') as 'target type',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'range') as 'range',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'area') as 'area',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'unknown float 1') as 'unknown float 1',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'unknown float 2') as 'unknown float 2',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'unknown float 3') as 'unknown float 3',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 id') as 'effect 0 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 0') as 'effect 0 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 1') as 'effect 0 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 2') as 'effect 0 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 id') as 'effect 1 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 0') as 'effect 1 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 1') as 'effect 1 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 2') as 'effect 1 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 id') as 'effect 2 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 0') as 'effect 2 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 1') as 'effect 2 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 2') as 'effect 2 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 id') as 'effect 3 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 0') as 'effect 3 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 1') as 'effect 3 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 2') as 'effect 3 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 id') as 'effect 4 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 0') as 'effect 4 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 1') as 'effect 4 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 2') as 'effect 4 data 2',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'str') as 'str',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'def') as 'def',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'ats') as 'ats',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'adf') as 'adf',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'acc') as 'acc',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'eva') as 'eva',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'spd') as 'spd',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'mov') as 'mov',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'hp') as 'hp',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'ep') as 'ep',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'price') as 'price',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'carry limit') as 'carry limit',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'sort id') as 'sort id',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'unknown short') as 'unknown short',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'name') as 'name',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'description') as 'description',
    (select value from tbl where file = entry.file and [entry number] = entry.[entry number] and attribute = 'unknown data') as 'unknown data'
from tbl as entry
join files on files.id = entry.file
where entry.attribute = 'header name'
and value = 'item'",
        [],
    )
    .unwrap();

    tx.execute("drop view if exists magic;", []).unwrap();
    tx.execute(
        "create view magic as
select
    files.path as file,
    entry.[entry number] as 'entry number',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'id') as id,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'character restriction') as 'character restriction',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'flags') as 'flags',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'category') as 'category',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'type') as 'type',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'element') as 'element',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'juna_specific') as 'juna_specific',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'target type') as 'target type',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'target range') as 'target range',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'target size') as 'target size',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown float 1') as 'unknown float 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown float 2') as 'unknown float 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown float 3') as 'unknown float 3',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 id') as 'effect 0 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 0') as 'effect 0 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 1') as 'effect 0 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 2') as 'effect 0 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 id') as 'effect 1 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 0') as 'effect 1 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 1') as 'effect 1 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 2') as 'effect 1 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 id') as 'effect 2 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 0') as 'effect 2 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 1') as 'effect 2 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 2') as 'effect 2 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 id') as 'effect 3 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 0') as 'effect 3 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 1') as 'effect 3 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 2') as 'effect 3 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 id') as 'effect 4 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 0') as 'effect 4 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 1') as 'effect 4 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 2') as 'effect 4 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'cast delay') as 'cast delay',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'recovery delay') as 'recovery delay',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'cost') as 'cost',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unbalance bonus') as 'unbalance bonus',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'break bonus') as 'break bonus',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'level learn') as 'level learn',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'sort id') as 'sort id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'animation') as 'animation',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'name') as 'name',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'description') as 'description'
from tbl as entry
join files on files.id = entry.file
where entry.attribute = 'header name'
and entry.value = 'magic'",
        [],
    )
    .unwrap();

    tx.execute("drop view if exists magicbo;", []).unwrap();
    tx.execute(
        "create view magicbo as
select
    files.path as file,
    entry.[entry number] as 'entry number',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'id') as id,
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 id') as 'effect 0 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 0') as 'effect 0 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 1') as 'effect 0 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 0 data 2') as 'effect 0 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 id') as 'effect 1 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 0') as 'effect 1 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 1') as 'effect 1 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 1 data 2') as 'effect 1 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 id') as 'effect 2 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 0') as 'effect 2 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 1') as 'effect 2 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 2 data 2') as 'effect 2 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 id') as 'effect 3 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 0') as 'effect 3 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 1') as 'effect 3 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 3 data 2') as 'effect 3 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 id') as 'effect 4 id',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 0') as 'effect 4 data 0',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 1') as 'effect 4 data 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'effect 4 data 2') as 'effect 4 data 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'turns') as 'turns'
from tbl as entry
join files on files.id = entry.file
where entry.attribute = 'header name'
and entry.value = 'magicbo'",
        [],
    )
    .unwrap();

    tx.execute("drop view if exists status;", []).unwrap();
    tx.execute(
        "create view status as
select
    files.path as file,
    entry.[entry number] as 'entry number',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'script') as 'script',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'texture') as 'texture',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'model') as 'model',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'model_scale') as 'model_scale',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'camera_pivot_height') as 'camera_pivot_height',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'hitbox_size') as 'hitbox_size',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown_float_1') as 'unknown_float_1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown_float_2') as 'unknown_float_2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown_float_3') as 'unknown_float_3',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown_float_4') as 'unknown_float_4',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown_short_1') as 'unknown_short_1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown_short_2') as 'unknown_short_2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'is_female') as 'is_female',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'level') as 'level',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'hp_base') as 'hp_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'hp_growth') as 'hp_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'max_ep') as 'max_ep',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'start_ep') as 'start_ep',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'max_cp') as 'max_cp',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'start_cp') as 'start_cp',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'str_base') as 'str_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'str_growth') as 'str_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'def_base') as 'def_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'def_growth') as 'def_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'ats_base') as 'ats_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'ats_growth') as 'ats_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'adf_base') as 'adf_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'adf_growth') as 'adf_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'dex_base') as 'dex_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'dex_growth') as 'dex_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'agl_base') as 'agl_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'agl_growth') as 'agl_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'eva') as 'eva',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'spd_base') as 'spd_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'spd_growth') as 'spd_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'mov_base') as 'mov_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'mov_growth') as 'mov_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'exp_base') as 'exp_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'exp_growth') as 'exp_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'brk_mult_base') as 'brk_mult_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'brk_mult_growth') as 'brk_mult_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'earth_efficacy') as 'earth_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'water_efficacy') as 'water_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'fire_efficacy') as 'fire_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'wind_efficacy') as 'wind_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'time_efficacy') as 'time_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'space_efficacy') as 'space_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'mirage_efficacy') as 'mirage_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'psn_efficacy') as 'psn_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'seal_efficacy') as 'seal_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'mute_efficacy') as 'mute_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'blnd_efficacy') as 'blnd_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'slp_efficacy') as 'slp_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'burn_efficacy') as 'burn_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'frz_efficacy') as 'frz_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'petr_efficacy') as 'petr_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'fnt_efficacy') as 'fnt_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'conf_efficacy') as 'conf_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'chrm_efficacy') as 'chrm_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'dblw_efficacy') as 'dblw_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'nmr_efficacy') as 'nmr_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'dlay_efficacy') as 'dlay_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'vnsh_efficacy') as 'vnsh_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 's_dwn_efficacy') as 's_dwn_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'slash_efficacy') as 'slash_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'thrust_efficacy') as 'thrust_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'pierce_efficacy') as 'pierce_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'strike_efficacy') as 'strike_efficacy',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'earth_sepith_base') as 'earth_sepith_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'water_sepith_base') as 'water_sepith_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'fire_sepith_base') as 'fire_sepith_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'wind_sepith_base') as 'wind_sepith_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'time_sepith_base') as 'time_sepith_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'space_sepith_base') as 'space_sepith_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'mirage_sepith_base') as 'mirage_sepith_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'sepith_mass_base') as 'sepith_mass_base',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'earth_sepith_growth') as 'earth_sepith_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'water_sepith_growth') as 'water_sepith_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'fire_sepith_growth') as 'fire_sepith_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'wind_sepith_growth') as 'wind_sepith_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'time_sepith_growth') as 'time_sepith_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'space_sepith_growth') as 'space_sepith_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'mirage_sepith_growth') as 'mirage_sepith_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'sepith_mass_growth') as 'sepith_mass_growth',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'drop_id_1') as 'drop_id_1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'drop_chance_1') as 'drop_chance_1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'drop_id_2') as 'drop_id_2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'drop_chance_2') as 'drop_chance_2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown_float_5') as 'unknown_float_5',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown_float_6') as 'unknown_float_6',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'flags') as 'flags',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'name') as 'name',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'description') as 'description'
from tbl as entry
join files on files.id = entry.file
where entry.attribute = 'header name'
and entry.value = 'status'",
        [],
    )
    .unwrap();

    tx.execute("drop view if exists voice;", []).unwrap();
    tx.execute(
        "create view voice as
select
    files.path as file,
    entry.[entry number] as 'entry number',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown short 1') as 'unknown short 1',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown byte') as 'unknown byte',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'clip') as 'clip',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown short 2') as 'unknown short 2',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown float') as 'unknown float',
    (select value from tbl where file = entry.file and tbl.[entry number] = entry.[entry number] and attribute = 'unknown data') as 'unknown data'
from tbl as entry
join files on files.id = entry.file
where entry.attribute = 'header name'
and entry.value = 'voice'",
        [],
    )
    .unwrap();

    tx.execute("drop view if exists effects", []).unwrap();
    tx.execute(
        "create view effects as
select
    entry.file as file,
    entry.[entry number] as 'entry number',
    cast (substring(entry.attribute, 8, 1) as int) as 'effect number',
    value as id,
    (select value from tbl where tbl.file = entry.file and tbl.[entry number] = entry.[entry number] and tbl.attribute = 'effect ' || substring(entry.attribute, 8, 1) || ' data 0') as 'data 0',
    (select value from tbl where tbl.file = entry.file and tbl.[entry number] = entry.[entry number] and tbl.attribute = 'effect ' || substring(entry.attribute, 8, 1) || ' data 1') as 'data 1',
    (select value from tbl where tbl.file = entry.file and tbl.[entry number] = entry.[entry number] and tbl.attribute = 'effect ' || substring(entry.attribute, 8, 1) || ' data 2') as 'data 2'
from tbl as entry
where attribute like 'effect % id'",
        [],
    )
    .unwrap();

    // insert data
    {
        let mut insert = tx.prepare_cached("insert into tbl (file, [entry number], attribute, value) values (?1, ?2, ?3, ?4)").unwrap();

        let cwd = std::env::current_dir().unwrap();
        let base_path = cwd.join(std::path::Path::new("private-test-data/cs3/data/text/"));
        for (file_id, direntry) in walkdir::WalkDir::new(base_path).into_iter().enumerate() {
            let direntry = direntry.unwrap();
            let path = direntry.path();
            if direntry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("tbl")) {
                println!("{}", path.display());
                let path_components: std::vec::Vec<_> = path.components().collect();
                let dbpath = (path_components.split_at(path_components.len() - 4).1)
                    .iter()
                    .fold(std::path::PathBuf::new(), |a, b| a.join(b.as_os_str()));
                tx.execute("insert into files (id, path) values (?1, ?2)", rusqlite::params![file_id, dbpath.display().to_string()])
                    .unwrap();
                let data = cs3_tbl::fix(&std::fs::read(path).unwrap());
                let tbl = cs3_tbl::load(&data).unwrap();
                for (n, tblentry) in tbl.entries.iter().enumerate() {
                    insert.execute(rusqlite::params![file_id, n, "header name", tblentry.header_name()]).unwrap();
                    match tblentry {
                        CS3Entry::Generic(cs3_entry_data::Generic { header: _, data }) => {
                            insert.execute(rusqlite::params![file_id, n, "data", data]).unwrap();
                        }
                        CS3Entry::CharRevise(cs3_entry_data::CharRevise {
                            char,
                            hp,
                            str,
                            def,
                            ats,
                            adf,
                            spd,
                            brk,
                        }) => {
                            insert.execute(rusqlite::params![file_id, n, "char", char]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "hp", hp]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "str", str]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "def", def]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "ats", ats]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "adf", adf]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "spd", spd]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "brk", brk]).unwrap();
                        }
                        CS3Entry::CompHelpData(cs3_entry_data::CompHelpData { r#type, text, value }) => {
                            insert.execute(rusqlite::params![file_id, n, "type", r#type]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "text", text]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "value", value]).unwrap();
                        }
                        CS3Entry::Item(cs3_entry_data::Item {
                            id,
                            character_restriction,
                            flags,
                            r#type,
                            subtype,
                            element,
                            slash,
                            thrust,
                            pierce,
                            strike,
                            unknown_byte,
                            target_type,
                            range,
                            area,
                            unknown_float_1,
                            unknown_float_2,
                            unknown_float_3,
                            effects,
                            str,
                            def,
                            ats,
                            adf,
                            acc,
                            eva,
                            spd,
                            mov,
                            hp,
                            ep,
                            price,
                            carry_limit,
                            sort_id,
                            unknown_short,
                            name,
                            description,
                            unknown_data,
                        }) => {
                            insert.execute(rusqlite::params![file_id, n, "id", id]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "character restriction", character_restriction]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "flags", flags]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "type", r#type]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "subtype", subtype]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "element", element]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "slash", slash]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "thrust", thrust]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "pierce", pierce]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "strike", strike]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown byte", unknown_byte]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "target type", target_type]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "range", range]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "area", area]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown float 1", unknown_float_1]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown float 2", unknown_float_2]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown float 3", unknown_float_3]).unwrap();
                            for (i, eff) in effects.iter().enumerate() {
                                insert.execute(rusqlite::params![file_id, n, format!("effect {i} id"), eff.id]).unwrap();
                                for (j, x) in eff.data.iter().enumerate() {
                                    insert.execute(rusqlite::params![file_id, n, format!("effect {i} data {j}"), x,]).unwrap();
                                }
                            }
                            insert.execute(rusqlite::params![file_id, n, "str", str]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "def", def]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "ats", ats]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "adf", adf]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "acc", acc]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "eva", eva]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "spd", spd]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "mov", mov]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "hp", hp]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "ep", ep]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "price", price]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "carry limit", carry_limit]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "sort id", sort_id]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown short", unknown_short]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "name", name]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "description", description]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown data", unknown_data]).unwrap();
                        }
                        CS3Entry::Magic(cs3_entry_data::Magic {
                            id,
                            character_restriction,
                            flags,
                            category,
                            r#type,
                            element,
                            juna_specific,
                            target_type,
                            target_range,
                            target_size,
                            unknown_float_1,
                            unknown_float_2,
                            unknown_float_3,
                            effects,
                            cast_delay,
                            recovery_delay,
                            cost,
                            unbalance_bonus,
                            break_bonus,
                            level_learn,
                            sort_id,
                            animation,
                            name,
                            description,
                        }) => {
                            insert.execute(rusqlite::params![file_id, n, "id", id,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "character restriction", character_restriction,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "flags", flags,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "category", category,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "type", r#type,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "element", element,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "juna specific", juna_specific,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "target type", target_type,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "target range", target_range,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "target size", target_size,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown float 1", unknown_float_1,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown float 2", unknown_float_2,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown float 3", unknown_float_3,]).unwrap();
                            for (i, eff) in effects.iter().enumerate() {
                                insert.execute(rusqlite::params![file_id, n, format!("effect {i} id"), eff.id]).unwrap();
                                for (j, x) in eff.data.iter().enumerate() {
                                    insert.execute(rusqlite::params![file_id, n, format!("effect {i} data {j}"), x,]).unwrap();
                                }
                            }
                            insert.execute(rusqlite::params![file_id, n, "cast_delay", cast_delay,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "recovery_delay", recovery_delay,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "cost", cost,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unbalance bonus", unbalance_bonus,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "break bonus", break_bonus,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "level learn", level_learn,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "sort id", sort_id,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "animation", animation,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "name", name,]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "description", description,]).unwrap();
                        }
                        CS3Entry::MagicBo(cs3_entry_data::MagicBo { id, effects, turns }) => {
                            insert.execute(rusqlite::params![file_id, n, "id", id,]).unwrap();
                            for (i, eff) in effects.iter().enumerate() {
                                insert.execute(rusqlite::params![file_id, n, format!("effect {i} id"), eff.id]).unwrap();
                                for (j, x) in eff.data.iter().enumerate() {
                                    insert.execute(rusqlite::params![file_id, n, format!("effect {i} data {j}"), x,]).unwrap();
                                }
                            }
                            insert.execute(rusqlite::params![file_id, n, "turns", turns,]).unwrap();
                        }
                        CS3Entry::SlotCost(cs3_entry_data::SlotCost {
                            slot_position,
                            level,
                            earth,
                            water,
                            fire,
                            wind,
                            time,
                            space,
                            mirage,
                        }) => {
                            insert.execute(rusqlite::params![file_id, n, "slot position", slot_position]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "level", level]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "earth", earth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "water", water]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "fire", fire]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "wind", wind]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "time", time]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "space", space]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "mirage", mirage]).unwrap();
                        }
                        CS3Entry::Status(cs3_entry_data::Status {
                            script,
                            texture,
                            model,
                            model_scale,
                            camera_pivot_height,
                            hitbox_size,
                            unknown_float_1,
                            unknown_float_2,
                            unknown_float_3,
                            unknown_float_4,
                            unknown_short_1,
                            unknown_short_2,
                            is_female,
                            level,
                            hp_base,
                            hp_growth,
                            max_ep,
                            start_ep,
                            max_cp,
                            start_cp,
                            str_base,
                            str_growth,
                            def_base,
                            def_growth,
                            ats_base,
                            ats_growth,
                            adf_base,
                            adf_growth,
                            dex_base,
                            dex_growth,
                            agl_base,
                            agl_growth,
                            eva,
                            spd_base,
                            spd_growth,
                            mov_base,
                            mov_growth,
                            exp_base,
                            exp_growth,
                            brk_mult_base,
                            brk_mult_growth,
                            earth_efficacy,
                            water_efficacy,
                            fire_efficacy,
                            wind_efficacy,
                            time_efficacy,
                            space_efficacy,
                            mirage_efficacy,
                            psn_efficacy,
                            seal_efficacy,
                            mute_efficacy,
                            blnd_efficacy,
                            slp_efficacy,
                            burn_efficacy,
                            frz_efficacy,
                            petr_efficacy,
                            fnt_efficacy,
                            conf_efficacy,
                            chrm_efficacy,
                            dblw_efficacy,
                            nmr_efficacy,
                            dlay_efficacy,
                            vnsh_efficacy,
                            s_dwn_efficacy,
                            slash_efficacy,
                            thrust_efficacy,
                            pierce_efficacy,
                            strike_efficacy,
                            earth_sepith_base,
                            water_sepith_base,
                            fire_sepith_base,
                            wind_sepith_base,
                            time_sepith_base,
                            space_sepith_base,
                            mirage_sepith_base,
                            sepith_mass_base,
                            earth_sepith_growth,
                            water_sepith_growth,
                            fire_sepith_growth,
                            wind_sepith_growth,
                            time_sepith_growth,
                            space_sepith_growth,
                            mirage_sepith_growth,
                            sepith_mass_growth,
                            drop_id_1,
                            drop_chance_1,
                            drop_id_2,
                            drop_chance_2,
                            unknown_float_5,
                            unknown_float_6,
                            flags,
                            name,
                            description,
                        }) => {
                            insert.execute(rusqlite::params![file_id, n, "script", script]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "texture", texture]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "model", model]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "model_scale", model_scale]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "camera_pivot_height", camera_pivot_height]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "hitbox_size", hitbox_size]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown_float_1", unknown_float_1]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown_float_2", unknown_float_2]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown_float_3", unknown_float_3]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown_float_4", unknown_float_4]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown_short_1", unknown_short_1]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown_short_2", unknown_short_2]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "is_female", is_female]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "level", level]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "hp_base", hp_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "hp_growth", hp_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "max_ep", max_ep]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "start_ep", start_ep]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "max_cp", max_cp]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "start_cp", start_cp]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "str_base", str_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "str_growth", str_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "def_base", def_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "def_growth", def_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "ats_base", ats_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "ats_growth", ats_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "adf_base", adf_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "adf_growth", adf_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "dex_base", dex_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "dex_growth", dex_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "agl_base", agl_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "agl_growth", agl_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "eva", eva]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "spd_base", spd_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "spd_growth", spd_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "mov_base", mov_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "mov_growth", mov_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "exp_base", exp_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "exp_growth", exp_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "brk_mult_base", brk_mult_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "brk_mult_growth", brk_mult_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "earth_efficacy", earth_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "water_efficacy", water_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "fire_efficacy", fire_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "wind_efficacy", wind_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "time_efficacy", time_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "space_efficacy", space_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "mirage_efficacy", mirage_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "psn_efficacy", psn_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "seal_efficacy", seal_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "mute_efficacy", mute_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "blnd_efficacy", blnd_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "slp_efficacy", slp_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "burn_efficacy", burn_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "frz_efficacy", frz_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "petr_efficacy", petr_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "fnt_efficacy", fnt_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "conf_efficacy", conf_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "chrm_efficacy", chrm_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "dblw_efficacy", dblw_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "nmr_efficacy", nmr_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "dlay_efficacy", dlay_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "vnsh_efficacy", vnsh_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "s_dwn_efficacy", s_dwn_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "slash_efficacy", slash_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "thrust_efficacy", thrust_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "pierce_efficacy", pierce_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "strike_efficacy", strike_efficacy]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "earth_sepith_base", earth_sepith_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "water_sepith_base", water_sepith_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "fire_sepith_base", fire_sepith_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "wind_sepith_base", wind_sepith_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "time_sepith_base", time_sepith_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "space_sepith_base", space_sepith_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "mirage_sepith_base", mirage_sepith_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "sepith_mass_base", sepith_mass_base]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "earth_sepith_growth", earth_sepith_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "water_sepith_growth", water_sepith_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "fire_sepith_growth", fire_sepith_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "wind_sepith_growth", wind_sepith_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "time_sepith_growth", time_sepith_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "space_sepith_growth", space_sepith_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "mirage_sepith_growth", mirage_sepith_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "sepith_mass_growth", sepith_mass_growth]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "drop_id_1", drop_id_1]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "drop_chance_1", drop_chance_1]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "drop_id_2", drop_id_2]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "drop_chance_2", drop_chance_2]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown_float_5", unknown_float_5]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown_float_6", unknown_float_6]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "flags", flags]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "name", name]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "description", description]).unwrap();
                        }
                        CS3Entry::Voice(cs3_entry_data::Voice {
                            unknown_short_1,
                            unknown_byte,
                            clip,
                            unknown_short_2,
                            unknown_float,
                            unknown_data,
                        }) => {
                            insert.execute(rusqlite::params![file_id, n, "unknown short 1", unknown_short_1]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown byte", unknown_byte]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "clip", clip]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown short 2", unknown_short_2]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown float", unknown_float]).unwrap();
                            insert.execute(rusqlite::params![file_id, n, "unknown data", unknown_data]).unwrap();
                        }
                    }
                }
            }
        }
    }

    tx.commit().unwrap();

    // test statements for statistics, to (hopefully) get faster interactive queries
    let _ = db
        .query_row("select header, count(*) as [count] from headers group by header order by [count] limit 1", [], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
        })
        .unwrap();
    let _ = db.query_row("select count(*), max(length(data)) from generic_entries", [], |row| row.get::<_, usize>(0)).unwrap();
    let _ = db
        .query_row("select count(*) from char_revise where file like '%/dat_en/%' and hp <> 100", [], |row| row.get::<_, usize>(0))
        .unwrap();
    let _ = db.query_row("select count(*) from item", [], |row| row.get::<_, usize>(0)).unwrap();
    let _ = db.query_row("select count(*) from (select distinct id from effects)", [], |row| row.get::<_, usize>(0)).unwrap();

    let _ = db
        .query_row(
            "select files.path as 'file', tbl.'entry number' as 'entry number', tbl.value as 'data'
from tbl
join files on tbl.file = files.id
join (
    select distinct tbl.file as file, tbl.'entry number' as 'entry number'
    from tbl
    where attribute = 'header name'
    and value = 'QSChapter'
) as tbl2 on tbl2.file = tbl.file and tbl2.'entry number' = tbl.'entry number'
where tbl.attribute = 'data'
order by files.path, tbl.'entry number'
limit 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?, row.get::<_, std::vec::Vec<u8>>(2)?)),
        )
        .unwrap();

    let _ = db
        .query_row(
            "select files.path, [header name], count(*)
from generic_entries
join files on generic_entries.file = files.id
where files.path like '%/dat_en/%'
group by [header name]
order by count(*) desc
limit 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, usize>(2)?)),
        )
        .unwrap();

    db.execute("pragma optimize;", []).unwrap();
}
