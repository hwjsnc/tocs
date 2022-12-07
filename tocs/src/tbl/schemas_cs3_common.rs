[
    (SmolStr::new("effect"), [
        ("id", Type::U16),
        ("data", Type::Repeat(3, Box::new(Type::I32))),
    ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
    (SmolStr::new("QSCookVariantDescription"), [
        ("item_id", Type::I16),
        ("description_1", Type::CUtf8),
        ("description_2", Type::CUtf8),
    ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
].into_iter().map(|(hdr, schema)| (Header::new(&hdr).unwrap(), schema)).collect()
