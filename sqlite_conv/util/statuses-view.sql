create view statuses as
select script, name, "psn" as status, psn_efficacy as efficacy from status
union all
select script, name, "seal" as status, seal_efficacy as efficacy from status
union all
select script, name, "mute" as status, mute_efficacy as efficacy from status
union all
select script, name, "blnd" as status, blnd_efficacy as efficacy from status
union all
select script, name, "slp" as status, slp_efficacy as efficacy from status
union all
select script, name, "burn" as status, burn_efficacy as efficacy from status
union all
select script, name, "frz" as status, frz_efficacy as efficacy from status
union all
select script, name, "petr" as status, petr_efficacy as efficacy from status
union all
select script, name, "fnt" as status, fnt_efficacy as efficacy from status
union all
select script, name, "conf" as status, conf_efficacy as efficacy from status
union all
select script, name, "chrm" as status, chrm_efficacy as efficacy from status
union all
select script, name, "dblw" as status, dblw_efficacy as efficacy from status
union all
select script, name, "nmr" as status, nmr_efficacy as efficacy from status
union all
select script, name, "dlay" as status, dlay_efficacy as efficacy from status
union all
select script, name, "vnsh" as status, vnsh_efficacy as efficacy from status
;