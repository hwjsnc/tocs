[
    (SmolStr::new("effect"), [
        ("id", Type::U16),
        ("data", Type::Repeat(3, Box::new(Type::I32))),
    ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
    (SmolStr::new("QSCook_ingredient"), [
        ("id", Type::U16),
        ("count", Type::U16),
    ].into_iter().map(|(s, t)| (SmolStr::new(s), t)).collect()),
].into_iter().map(|(hdr, schema)| (Header::new(&hdr).unwrap(), schema)).collect()
