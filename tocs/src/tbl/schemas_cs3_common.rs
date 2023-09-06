&[
    ("dlc_item", &[
        ("id", Type::U16),
        ("count", Type::U16),
    ]),
    ("effect", &[
        ("id", Type::U16),
        ("data", Type::Repeat(3, Box::new(Type::I32))),
    ]),
    ("QSCookVariantDescription", &[
        ("item_id", Type::I16),
        ("description_1", Type::CUtf8),
        ("description_2", Type::CUtf8),
    ]),
]
