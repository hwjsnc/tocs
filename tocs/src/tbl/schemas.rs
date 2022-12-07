use smol_str::SmolStr; // TODO: remove

use super::{get_nf1_for_all, AtomicSchemas, Schemas, Type};

use super::Header;

lazy_static::lazy_static! {
    pub static ref CS1 : Schemas = gen_schemas_cs1();
    pub static ref CS1_COMMON : Schemas = gen_schemas_cs1_common();
    pub static ref CS2 : Schemas = gen_schemas_cs2();
    pub static ref CS2_COMMON : Schemas = gen_schemas_cs2_common();
    pub static ref CS3 : Schemas = gen_schemas_cs3();
    pub static ref CS3_COMMON : Schemas = gen_schemas_cs3_common();
    pub static ref CS4 : Schemas = gen_schemas_cs4();
    pub static ref CS4_COMMON : Schemas = gen_schemas_cs4_common();
    pub static ref CS5 : Schemas = gen_schemas_cs5();
    pub static ref CS5_COMMON : Schemas = gen_schemas_cs5_common();

    pub static ref CS1_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS1, &CS1_COMMON).expect("");
    pub static ref CS2_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS2, &CS2_COMMON).expect("");
    pub static ref CS3_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS3, &CS3_COMMON).expect("");
    pub static ref CS4_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS4, &CS4_COMMON).expect("");
    pub static ref CS5_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS5, &CS5_COMMON).expect("");

    pub static ref EMPTY : AtomicSchemas = [].into_iter().collect();
}

fn gen_schemas_cs1() -> Schemas {
    include!("schemas_cs1.rs")
}

fn gen_schemas_cs1_common() -> Schemas {
    include!("schemas_cs1_common.rs")
}

fn gen_schemas_cs2() -> Schemas {
    include!("schemas_cs2.rs")
}

fn gen_schemas_cs2_common() -> Schemas {
    include!("schemas_cs2_common.rs")
}

fn gen_schemas_cs3() -> Schemas {
    include!("schemas_cs3.rs")
}

fn gen_schemas_cs3_common() -> Schemas {
    include!("schemas_cs3_common.rs")
}

fn gen_schemas_cs4() -> Schemas {
    include!("schemas_cs4.rs")
}

fn gen_schemas_cs4_common() -> Schemas {
    include!("schemas_cs4_common.rs")
}

fn gen_schemas_cs5() -> Schemas {
    include!("schemas_cs5.rs")
}

fn gen_schemas_cs5_common() -> Schemas {
    include!("schemas_cs5_common.rs")
}
