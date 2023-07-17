select
 status_p.id as id,
 status_p.name as character,
 str_base + 250 * str_growth as str,
 weapon.name as weapon_name,
 weapon.str as weapon_str,
 weapon.ats as weapon_ats,
 weapon.spd as weapon_spd,
 armour.name as armour_name,
 armour.str as armour_str,
 armour.ats as armour_ats,
 armour.spd as armour_spd,
 boots.name as boots_name,
 boots.str as boots_str,
 boots.ats as boots_ats,
 boots.spd as boots_spd,
 str_base + 250 * str_growth + weapon.str + armour.str + boots.str as total_str,
 ats_base + 250 * ats_growth + weapon.ats + armour.ats + boots.ats as total_ats,
 spd_base + 250 * spd_growth + weapon.spd + armour.spd + boots.spd as total_spd
from status_p
left join item_e as weapon on (weapon.type = 150 and weapon.character_restriction = status_p.id)
left join item_e as armour on (armour.type = 190 and (armour.character_restriction = status_p.id or armour.character_restriction = 65535))
left join item_e as boots on (boots.type = 191 and (boots.character_restriction = status_p.id or boots.character_restriction = 65535))
group by status_p.id
having total_str = max(total_str)
order by total_str desc
;