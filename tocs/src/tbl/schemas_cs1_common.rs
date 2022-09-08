[
    (SmolStr::new("effect"), [
        ("id", Type::U8),
        ("data", Type::Repeat(2, Box::new(Type::I16))),
    ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
    (SmolStr::new("QSCookVariantDescription"), [
        ("item_id", Type::I16),
        ("description_1", Type::CUtf8),
        ("description_2", Type::CUtf8),
    ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
].into_iter().collect()