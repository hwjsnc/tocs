use super::{try_atomic_value_from_str, AtomicSchemas, AtomicType, Entry, Header, Schemas, Tbl, VersionWithoutSchema};

pub fn schemas_to_json(schemas: &Schemas) -> String {
    // TODO: get rid of serde
    serde_json::to_string_pretty(schemas).unwrap()
}

pub fn try_schemas_from_json(s: &[u8]) -> Result<Schemas, impl std::error::Error> {
    serde_json::from_slice(s)
}

pub fn tbl_to_json(tbl: &Tbl) -> String {
    use serde_json::value::Value;
    let version = match tbl.version {
        VersionWithoutSchema::V1 => Value::String("v1".to_owned()),
        VersionWithoutSchema::V2 => Value::String("v2".to_owned()),
    };
    let headers = Value::Array(tbl.headers.iter().map(|h| Value::String(h.as_str().to_owned())).collect());
    let entries = Value::Array(
        tbl.entries
            .iter()
            .map(|ent| {
                let values = Value::Object(ent.values.iter().map(|(attr, val)| (attr.as_str().to_owned(), Value::String(val.to_string()))).collect());
                Value::Object(
                    [("header".to_owned(), Value::String(ent.header.as_str().to_owned())), ("values".to_owned(), values)]
                        .into_iter()
                        .collect(),
                )
            })
            .collect(),
    );
    let obj: indexmap::IndexMap<_, _> = [("version", version), ("headers", headers), ("entries", entries)].into_iter().collect();
    serde_json::to_string_pretty(&obj).unwrap()
}

pub fn try_tbl_from_json(schemas: &AtomicSchemas, bytes: &[u8]) -> Result<Tbl, impl std::error::Error> {
    use serde_json::Value;

    #[derive(Debug)]
    struct Error(String);
    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for Error {}

    let v: Value = serde_json::from_slice(bytes).map_err(|err| Error(format!("invalid JSON: {err}")))?;
    let Value::Object(outer) = v else { return Err(Error("outer value is not a map".to_owned())) };

    let version = match outer.get("version") {
        Some(Value::String(s)) if s == "v1" => VersionWithoutSchema::V1,
        Some(Value::String(s)) if s == "v2" => VersionWithoutSchema::V2,
        Some(val) => return Err(Error(format!("invalid schema version {val}"))),
        None => return Err(Error("missing schema version".to_owned())),
    };

    let headers = match outer.get("headers") {
        Some(Value::Array(arr)) => {
            let mut headers = std::vec::Vec::with_capacity(arr.len());
            for hdr in arr.iter() {
                let Value::String(s) = &hdr else { return Err(Error(format!("not a valid header: {hdr}"))) };
                let h = Header::new(s.as_str()).map_err(|err| Error(format!("not a valid header name: '{hdr}', {err}")))?;
                headers.push(h);
            }
            headers
        }
        Some(val) => return Err(Error(format!("invalid header list: {val}"))),
        None => return Err(Error("missing header list".to_owned())),
    };

    let arr = match outer.get("entries") {
        Some(Value::Array(arr)) => arr,
        Some(val) => return Err(Error(format!("not a valid list of entries: {val}"))),
        None => return Err(Error("missing list of entries".to_owned())),
    };
    let mut entries = std::vec::Vec::with_capacity(arr.len());
    for (i, ent) in arr.iter().enumerate() {
        let i = i + 1;
        let ent_map = match ent {
            Value::Object(m) => m,
            _ => return Err(Error(format!("unexpected entry {i}, should be an object: {ent}"))),
        };
        let header = match ent_map.get("header") {
            Some(Value::String(s)) => Header::new(s.as_str()).map_err(|err| Error(format!("entry {i}: invalid header name: '{s}', {err}")))?,
            Some(val) => return Err(Error(format!("entry{i}: invalid entry header name: {val}"))),
            None => return Err(Error(format!("entry {i}: missing header name"))),
        };
        let value_map = match ent_map.get("values") {
            Some(Value::Object(m)) => m,
            Some(val) => return Err(Error(format!("entry {i}: invalid values: {val}"))),
            None => return Err(Error(format!("entry {i}: missing values"))),
        };
        let values = match schemas.get(&header) {
            Some(schema) => {
                let mut values = indexmap::IndexMap::with_capacity(schema.len());
                for (attr, ty) in schema.iter() {
                    let Some(v) = value_map.get(attr.as_str()) else {
                        return Err(Error(format!("entry {i}: missing attribute {attr}")));
                    };
                    let Value::String(s) = v else {
                        return Err(Error(format!("entry {i}, attribute {attr}: value has to be a string")));
                    };
                    let x = try_atomic_value_from_str(ty, s.as_str(), /*strict = */ false).map_err(|err| Error(format!("entry {i}, attribute {attr}: invalid value {s} for type {ty:?}: {err}")))?;
                    values.insert(attr.clone(), x);
                }
                values
            }
            None => {
                let Some(v) = value_map.get("data") else { return Err(Error(format!("entry {i}: missing data"))) };
                let Value::String(s) = v else {
                    return Err(Error(format!("entry {i}, attribute data: value has to be a string")));
                };
                let ty = AtomicType::X(s.len() / 2);
                let x = try_atomic_value_from_str(&ty, s.as_str(), /*strict = */ false).map_err(|err| Error(format!("entry {i}: invalid value {s} for data: {err}")))?;
                [("data".into(), x)].into_iter().collect()
            }
        };
        entries.push(Entry { header, values });
    }

    Ok(Tbl::new_2(version, headers, entries))
}

#[cfg(test)]
mod test {
    use super::{schemas_to_json, tbl_to_json, try_schemas_from_json, try_tbl_from_json};
    use crate::tbl::{serialize, AtomicSchemas, AtomicType, AtomicValue, Attribute, Entry, Header, Schema, Schemas, Tbl, Type};
    use indexmap::IndexMap;

    #[test]
    fn test_schemas_json() {
        let s = br#"{
            "entries": {
                "magic": {
                    "id": "u16",
                    "unk_float": "f32",
                    "unk_data": {
                        "bytes": {
                            "count": 5
                        }
                    },
                    "name": "c_utf8"
                },
                "foo": {
                    "int1": "i32",
                    "int2": "i32",
                    "unk_byte": "u8",
                    "items": {
                        "repeat": {
                            "count": 5,
                            "type": {
                                "ref": "item_entry"
                            }
                        }
                    }
                }
            },
            "common": {
                "item_entry": {
                    "id": "i16",
                    "count": "u32"
                }
            }
        }"#;
        let entries: IndexMap<Header, Schema> = [
            (
                Header::new("magic").unwrap(),
                [
                    (Attribute::from("id"), Type::U16),
                    (Attribute::from("unk_float"), Type::F32),
                    (Attribute::from("unk_data"), Type::X(5)),
                    (Attribute::from("name"), Type::CUtf8),
                ]
                .into_iter()
                .collect(),
            ),
            (
                Header::new("foo").unwrap(),
                [
                    (Attribute::from("int1"), Type::I32),
                    (Attribute::from("int2"), Type::I32),
                    (Attribute::from("unk_byte"), Type::U8),
                    (Attribute::from("items"), Type::Repeat(5, Box::new(Type::Ref(Header::new("item_entry").unwrap())))),
                ]
                .into_iter()
                .collect(),
            ),
        ]
        .into_iter()
        .collect();
        let common: IndexMap<Header, Schema> = [(
            Header::new("item_entry").unwrap(),
            [(Attribute::from("id"), Type::I16), (Attribute::from("count"), Type::U32)].into_iter().collect(),
        )]
        .into_iter()
        .collect();

        let schemas = try_schemas_from_json(s).unwrap();

        assert_eq!(schemas.entries, entries);
        assert_eq!(schemas.common, common);

        assert!(crate::tbl::get_nf1_for_all(&schemas).is_ok());
    }

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
            let json = schemas_to_json(schemas);
            let copy: Schemas = try_schemas_from_json(json.as_bytes()).unwrap();
            assert_eq!(copy.entries, schemas.entries);
            assert_eq!(copy.common, schemas.common);
            assert!(crate::tbl::get_nf1_for_all(&copy).is_ok());
        }
    }

    #[test]
    fn test_tbl_json() {
        let schemas: AtomicSchemas = [
            (
                Header::new("MasterQuartzBase").unwrap(),
                [
                    ("item_id".into(), AtomicType::U16),
                    ("mq_id".into(), AtomicType::U16),
                    ("unknown_data".into(), AtomicType::X(4)),
                    ("scale".into(), AtomicType::F32),
                ]
                .into_iter()
                .collect(),
            ),
            (
                Header::new("MasterQuartzData").unwrap(),
                [("item_id".into(), AtomicType::U16), ("level".into(), AtomicType::I16), ("text".into(), AtomicType::CUtf8)]
                    .into_iter()
                    .collect(),
            ),
        ]
        .into_iter()
        .collect();

        let tbl = Tbl::new_2(
            crate::tbl::VersionWithoutSchema::V2,
            ["MasterQuartzBase", "MasterQuartzData"].into_iter().map(|s| Header::new(s).unwrap()).collect(),
            [
                Entry {
                    header: Header::new("MasterQuartzBase").unwrap(),
                    values: [
                        ("item_id".into(), AtomicValue::U16(3200)),
                        ("mq_id".into(), AtomicValue::U16(5)),
                        ("unknown_data".into(), AtomicValue::X(hexhex::hex_literal!("8badf00d").as_slice().into())),
                        ("scale".into(), AtomicValue::F32(17.3)),
                    ]
                    .into_iter()
                    .collect(),
                },
                Entry {
                    header: Header::new("MasterQuartzData").unwrap(),
                    values: [
                        ("item_id".into(), AtomicValue::U16(3200)),
                        ("level".into(), AtomicValue::I16(1)),
                        ("text".into(), AtomicValue::CUtf8("3200 lv 1".into())),
                    ]
                    .into_iter()
                    .collect(),
                },
                Entry {
                    header: Header::new("MasterQuartzData").unwrap(),
                    values: [
                        ("item_id".into(), AtomicValue::U16(3200)),
                        ("level".into(), AtomicValue::I16(2)),
                        ("text".into(), AtomicValue::CUtf8("3200 lv 2".into())),
                    ]
                    .into_iter()
                    .collect(),
                },
                Entry {
                    header: Header::new("MasterQuartzBase").unwrap(),
                    values: [
                        ("item_id".into(), AtomicValue::U16(3201)),
                        ("mq_id".into(), AtomicValue::U16(6)),
                        ("unknown_data".into(), AtomicValue::X(hexhex::hex_literal!("badcab1e").as_slice().into())),
                        ("scale".into(), AtomicValue::F32(12.5)),
                    ]
                    .into_iter()
                    .collect(),
                },
                Entry {
                    header: Header::new("MasterQuartzData").unwrap(),
                    values: [
                        ("item_id".into(), AtomicValue::U16(3201)),
                        ("level".into(), AtomicValue::I16(1)),
                        ("text".into(), AtomicValue::CUtf8("3201 lv 1".into())),
                    ]
                    .into_iter()
                    .collect(),
                },
                Entry {
                    header: Header::new("MasterQuartzData").unwrap(),
                    values: [
                        ("item_id".into(), AtomicValue::U16(3201)),
                        ("level".into(), AtomicValue::I16(2)),
                        ("text".into(), AtomicValue::CUtf8("3201 lv 2".into())),
                    ]
                    .into_iter()
                    .collect(),
                },
            ]
            .into_iter()
            .collect(),
        );

        let s = br#"{
            "version": "v2",
            "headers": ["MasterQuartzBase", "MasterQuartzData"],
            "entries": [
                {
                    "header": "MasterQuartzBase",
                    "values": {
                        "item_id": "3200",
                        "mq_id": "5",
                        "unknown_data": "8badf00d",
                        "scale": "17.3"
                    }
                },
                {
                    "header": "MasterQuartzData",
                    "values": {
                        "item_id": "3200",
                        "level": "1",
                        "text": "3200 lv 1"
                    }
                },
                {
                    "header": "MasterQuartzData",
                    "values": {
                        "item_id": "3200",
                        "level": "2",
                        "text": "3200 lv 2"
                    }
                },
                {
                    "header": "MasterQuartzBase",
                    "values": {
                        "item_id": "3201",
                        "mq_id": "6",
                        "unknown_data": "badcab1e",
                        "scale": "12.5"
                    }
                },
                {
                    "header": "MasterQuartzData",
                    "values": {
                        "item_id": "3201",
                        "level": "1",
                        "text": "3201 lv 1"
                    }
                },
                {
                    "header": "MasterQuartzData",
                    "values": {
                        "item_id": "3201",
                        "level": "2",
                        "text": "3201 lv 2"
                    }
                }
            ]
        }"#;

        let parsed_tbl = try_tbl_from_json(&schemas, s).unwrap();

        assert_eq!(parsed_tbl.version, tbl.version);
        assert_eq!(parsed_tbl.headers, tbl.headers);
        assert_eq!(parsed_tbl.entries.len(), tbl.entries.len());

        for (i, (ent1, ent2)) in parsed_tbl.entries.iter().zip(tbl.entries.iter()).enumerate() {
            println!("entry {i}");
            assert_eq!(ent1.header, ent2.header);
            assert_eq!(ent1.values.len(), ent2.values.len());
            for (j, ((attr1, val1), (attr2, val2))) in ent1.values.iter().zip(ent2.values.iter()).enumerate() {
                println!("attribute {j}");
                assert_eq!(attr1, attr2);
                match (val1, val2) {
                    (AtomicValue::I8(a), AtomicValue::I8(b)) if a == b => {}
                    (AtomicValue::I16(a), AtomicValue::I16(b)) if a == b => {}
                    (AtomicValue::I32(a), AtomicValue::I32(b)) if a == b => {}
                    (AtomicValue::U8(a), AtomicValue::U8(b)) if a == b => {}
                    (AtomicValue::U16(a), AtomicValue::U16(b)) if a == b => {}
                    (AtomicValue::U32(a), AtomicValue::U32(b)) if a == b => {}
                    (AtomicValue::F32(a), AtomicValue::F32(b)) if a == b => {}
                    (AtomicValue::CUtf8(a), AtomicValue::CUtf8(b)) if a == b => {}
                    (AtomicValue::X(a), AtomicValue::X(b)) if a == b => {}
                    _ => panic!("'{val1}' != '{val2}'"),
                }
            }
        }

        let data1 = serialize(&tbl).unwrap();
        let data2 = serialize(&parsed_tbl).unwrap();

        assert!(data1 == data2);

        // now check roundtrip
        let roundtripped_tbl = try_tbl_from_json(&schemas, tbl_to_json(&tbl).as_bytes()).unwrap();
        assert_eq!(serialize(&roundtripped_tbl).unwrap(), data1);
    }

    // TODO: maybe roundtrip test all tbls?
}
