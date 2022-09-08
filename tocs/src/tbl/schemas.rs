use indexmap::IndexMap;
use smol_str::SmolStr;

use super::{get_nf1_for_all, AtomicType, Type};

lazy_static::lazy_static! {
    pub static ref CS1 : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = gen_schemas_cs1();
    pub static ref CS1_COMMON : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = gen_schemas_cs1_common();
    pub static ref CS2 : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = gen_schemas_cs2();
    pub static ref CS2_COMMON : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = gen_schemas_cs2_common();
    pub static ref CS3 : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = gen_schemas_cs3();
    pub static ref CS3_COMMON : IndexMap<SmolStr, IndexMap<SmolStr, Type>> = gen_schemas_cs3_common();

    pub static ref CS1_ATOMIC : IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>> = get_nf1_for_all(&CS1, &CS1_COMMON).expect("");
    pub static ref CS2_ATOMIC : IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>> = get_nf1_for_all(&CS2, &CS2_COMMON).expect("");
    pub static ref CS3_ATOMIC : IndexMap<SmolStr, IndexMap<SmolStr, AtomicType>> = get_nf1_for_all(&CS3, &CS3_COMMON).expect("");
}

fn gen_schemas_cs1() -> IndexMap<SmolStr, IndexMap<SmolStr, Type>> {
    include!("schemas_cs1.rs")
}

fn gen_schemas_cs1_common() -> IndexMap<SmolStr, IndexMap<SmolStr, Type>> {
    include!("schemas_cs1_common.rs")
}

fn gen_schemas_cs2() -> IndexMap<SmolStr, IndexMap<SmolStr, Type>> {
    include!("schemas_cs2.rs")
}

fn gen_schemas_cs2_common() -> IndexMap<SmolStr, IndexMap<SmolStr, Type>> {
    include!("schemas_cs2_common.rs")
}

fn gen_schemas_cs3() -> IndexMap<SmolStr, IndexMap<SmolStr, Type>> {
    include!("schemas_cs3.rs")
}

fn gen_schemas_cs3_common() -> IndexMap<SmolStr, IndexMap<SmolStr, Type>> {
    include!("schemas_cs3_common.rs")
}
