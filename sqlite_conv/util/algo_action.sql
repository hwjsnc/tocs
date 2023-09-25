.open :memory:
attach database 'file:reverie-scripts.db?mode=ro' as scripts;
attach database 'file:reverie-1.0.6.db?mode=ro' as tbl;

.headers on
.mode csv
.once reverie_actions.csv

select
  ActionTable.file_name,
  (select enemy.name from tbl.status as enemy where ActionTable.file_name = 'battle/dat_en/al' || enemy.script || '.dat') as enemy_name,
  (select enemy.level from tbl.status as enemy where ActionTable.file_name = 'battle/dat_en/al' || enemy.script || '.dat') as enemy_level,
  ActionTable.name as action_name,
  ActionTable.id as action_id,
  ActionTable.category,
  ActionTable.target_type,
  ActionTable.target_range,
  ActionTable.target_size,
  ActionTable.flags as action_flags,
  unk4,
  unk5,
  unk6,
  cast_delay,
  recovery_delay,
  eff_1 as effect_1,
  param_1_1,
  param_1_2,
  param_1_3,
  eff_2 as effect_2,
  param_2_1,
  param_2_2,
  param_2_3,
  eff_3 as effect_3,
  param_3_1,
  param_3_2,
  param_3_3,
  eff_4 as effect_4,
  param_4_1,
  param_4_2,
  param_4_3,
  eff_5 as effect_5,
  param_5_1,
  param_5_2,
  param_5_3,
  unk18
from ActionTable
;

.once reverie_algo_table.csv

select
  AlgoTable.file_name,
  (select enemy.name from tbl.status as enemy where AlgoTable.file_name = 'battle/dat_en/al' || enemy.script || '.dat') as enemy_name,
  (select enemy.level from tbl.status as enemy where AlgoTable.file_name = 'battle/dat_en/al' || enemy.script || '.dat') as enemy_level,
  AlgoTable.action_or_magic_id,
  AlgoTable.condition,
  AlgoTable.cond_param_1,
  AlgoTable.cond_param_2,
  AlgoTable.cond_param_3,
  AlgoTable.chance,
  AlgoTable.use_limit,
  AlgoTable.target_priority,
  AlgoTable.target_param
from scripts.AlgoTable
;
