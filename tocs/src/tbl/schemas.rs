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

fn schemas_from_list(schemas_list: &[(&str, &[(&str, Type)])], common_list: &[(&str, &[(&str, Type)])]) -> Schemas {
    let mut entries = indexmap::IndexMap::with_capacity(schemas_list.len());
    for (header_name, attr_list) in schemas_list.iter() {
        let mut attrs = indexmap::IndexMap::with_capacity(attr_list.len());
        for (attr_name, attr_type) in attr_list.iter() {
            let prev = attrs.insert((*attr_name).into(), attr_type.clone());
            if prev.is_some() {
                panic!("duplicate attribute '{attr_name}' for '{header_name}'");
            }
        }
        let prev = entries.insert(Header::new(header_name).unwrap(), attrs);
        if prev.is_some() {
            panic!("duplicate schema for '{header_name}'");
        }
    }
    let mut common = indexmap::IndexMap::with_capacity(common_list.len());
    for (header_name, attr_list) in common_list.iter() {
        let name = Header::new(header_name).unwrap();
        if entries.contains_key(&name) {
            panic!("common schema '{header_name}' is also a schema name");
        }
        let mut attrs = indexmap::IndexMap::with_capacity(attr_list.len());
        for (attr_name, attr_type) in attr_list.iter() {
            let prev = attrs.insert((*attr_name).into(), attr_type.clone());
            if prev.is_some() {
                panic!("duplicate attribute '{attr_name}' for '{header_name}'");
            }
        }
        let prev = common.insert(name, attrs);
        if prev.is_some() {
            panic!("duplicate schema for common structure '{header_name}'");
        }
    }
    Schemas { entries, common }
}

fn gen_schemas_cs1() -> Schemas {
    schemas_from_list(include!("schemas_cs1.rs"), include!("schemas_cs1_common.rs"))
}

fn gen_schemas_cs2() -> Schemas {
    schemas_from_list(include!("schemas_cs2.rs"), include!("schemas_cs2_common.rs"))
}

fn gen_schemas_cs3() -> Schemas {
    schemas_from_list(include!("schemas_cs3.rs"), include!("schemas_cs3_common.rs"))
}

fn gen_schemas_cs4() -> Schemas {
    schemas_from_list(include!("schemas_cs4.rs"), include!("schemas_cs4_common.rs"))
}

fn gen_schemas_cs5() -> Schemas {
    schemas_from_list(include!("schemas_cs5.rs"), include!("schemas_cs5_common.rs"))
}
