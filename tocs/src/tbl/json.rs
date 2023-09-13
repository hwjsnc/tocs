use super::Schemas;

pub fn schemas_to_json(schemas: &Schemas) -> String {
    // TODO: get rid of serde
    serde_json::to_string_pretty(schemas).unwrap()
}

pub fn try_schemas_from_json(s: &[u8]) -> Result<Schemas, impl std::error::Error> {
    serde_json::from_slice(s)
}

#[cfg(test)]
mod test {
    use super::{schemas_to_json_string, try_schemas_from_json, Schemas};

    #[test]
    fn roundtrip_schemas_json() {
        let game_schemas: [&Schemas; 5] = [
            &crate::tbl::schemas::CS1 as &Schemas,
            &crate::tbl::schemas::CS2,
            &crate::tbl::schemas::CS3,
            &crate::tbl::schemas::CS4,
            &crate::tbl::schemas::CS4,
        ];

        for schemas in game_schemas {
            let json = schemas_to_json_string(schemas).unwrap();
            let copy: Schemas = try_schemas_from_json(json.as_bytes()).unwrap();
            assert_eq!(copy.entries, schemas.entries);
            assert_eq!(copy.common, schemas.common);
            assert!(super::get_nf1_for_all(&copy).is_ok());
        }
    }

    // TODO: json "hardcoded" schema check
}
