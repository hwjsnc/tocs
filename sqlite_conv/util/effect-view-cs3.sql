create view effects as
  select
    file,
    'item' as header,
    "effects[1] id" as 'effect id',
    "effects[1] data[1]" as 'data[1]',
    "effects[1] data[2]" as 'data[2]',
    "effects[1] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item
  union all
  select
    file,
    'item' as header,
    "effects[2] id" as 'effect id',
    "effects[2] data[1]" as 'data[1]',
    "effects[2] data[2]" as 'data[2]',
    "effects[2] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item
  union all
  select
    file,
    'item' as header,
    "effects[3] id" as 'effect id',
    "effects[3] data[1]" as 'data[1]',
    "effects[3] data[2]" as 'data[2]',
    "effects[3] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item
  union all
  select
    file,
    'item' as header,
    "effects[4] id" as 'effect id',
    "effects[4] data[1]" as 'data[1]',
    "effects[4] data[2]" as 'data[2]',
    "effects[4] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item
  union all
  select
    file,
    'item' as header,
    "effects[5] id" as 'effect id',
    "effects[5] data[1]" as 'data[1]',
    "effects[5] data[2]" as 'data[2]',
    "effects[5] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item
  union all
  select
    file,
    'item_q' as header,
    "effects[1] id" as 'effect id',
    "effects[1] data[1]" as 'data[1]',
    "effects[1] data[2]" as 'data[2]',
    "effects[1] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item_q
  union all
  select
    file,
    'item_q' as header,
    "effects[2] id" as 'effect id',
    "effects[2] data[1]" as 'data[1]',
    "effects[2] data[2]" as 'data[2]',
    "effects[2] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item_q
  union all
  select
    file,
    'item_q' as header,
    "effects[3] id" as 'effect id',
    "effects[3] data[1]" as 'data[1]',
    "effects[3] data[2]" as 'data[2]',
    "effects[3] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item_q
  union all
  select
    file,
    'item_q' as header,
    "effects[4] id" as 'effect id',
    "effects[4] data[1]" as 'data[1]',
    "effects[4] data[2]" as 'data[2]',
    "effects[4] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item_q
  union all
  select
    file,
    'item_q' as header,
    "effects[5] id" as 'effect id',
    "effects[5] data[1]" as 'data[1]',
    "effects[5] data[2]" as 'data[2]',
    "effects[5] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from item_q
  union all
  select
    file,
    'magic' as header,
    "effects[1] id" as 'effect id',
    "effects[1] data[1]" as 'data[1]',
    "effects[1] data[2]" as 'data[2]',
    "effects[1] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from magic
  union all
  select
    file,
    'magic' as header,
    "effects[2] id" as 'effect id',
    "effects[2] data[1]" as 'data[1]',
    "effects[2] data[2]" as 'data[2]',
    "effects[2] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from magic
  union all
  select
    file,
    'magic' as header,
    "effects[3] id" as 'effect id',
    "effects[3] data[1]" as 'data[1]',
    "effects[3] data[2]" as 'data[2]',
    "effects[3] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from magic
  union all
  select
    file,
    'magic' as header,
    "effects[4] id" as 'effect id',
    "effects[4] data[1]" as 'data[1]',
    "effects[4] data[2]" as 'data[2]',
    "effects[4] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from magic
  union all
  select
    file,
    'magic' as header,
    "effects[5] id" as 'effect id',
    "effects[5] data[1]" as 'data[1]',
    "effects[5] data[2]" as 'data[2]',
    "effects[5] data[3]" as 'data[3]',
    id as 'entry id',
    name as "entry name",
    description as "entry description"
  from magic
