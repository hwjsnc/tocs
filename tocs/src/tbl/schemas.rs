use smol_str::SmolStr; // TODO: remove

use super::{get_nf1_for_all, AtomicSchemas, Schemas, Type};

use super::Header;

lazy_static::lazy_static! {
    pub static ref CS1 : Schemas = gen_schemas_cs1();
    pub static ref CS2 : Schemas = gen_schemas_cs2();
    pub static ref CS3 : Schemas = gen_schemas_cs3();
    pub static ref CS4 : Schemas = gen_schemas_cs4();
    pub static ref CS5 : Schemas = gen_schemas_cs5();

    pub static ref CS1_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS1).expect("");
    pub static ref CS2_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS2).expect("");
    pub static ref CS3_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS3).expect("");
    pub static ref CS4_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS4).expect("");
    pub static ref CS5_ATOMIC : AtomicSchemas = get_nf1_for_all(&CS5).expect("");

    pub static ref EMPTY : AtomicSchemas = [].into_iter().collect();
}

fn gen_schemas_cs1() -> Schemas {
    Schemas {
        entries: include!("schemas_cs1.rs"),
        common: include!("schemas_cs1_common.rs"),
    }
}

fn gen_schemas_cs2() -> Schemas {
    Schemas {
        entries: include!("schemas_cs2.rs"),
        common: include!("schemas_cs2_common.rs"),
    }
}

fn gen_schemas_cs3() -> Schemas {
    Schemas {
        entries: include!("schemas_cs3.rs"),
        common: include!("schemas_cs3_common.rs"),
    }
}

fn gen_schemas_cs4() -> Schemas {
    Schemas {
        entries: include!("schemas_cs4.rs"),
        common: include!("schemas_cs4_common.rs"),
    }
}

fn gen_schemas_cs5() -> Schemas {
    Schemas {
        entries: include!("schemas_cs5.rs"),
        common: include!("schemas_cs5_common.rs"),
    }
}
