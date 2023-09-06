&[
    ("effect", &[
        ("id", Type::U8),
        ("data", Type::Repeat(3, Box::new(Type::I32))),
    ]),
    ("InfMonster", &[
        ("id", Type::CUtf8),
        ("chance", Type::U8),
    ]),
    ("QSCookVariantDescription", &[
        ("item_id", Type::I16),
        ("description_1", Type::CUtf8),
        ("description_2", Type::CUtf8),
    ]),
    ("QSCookIngredient", &[
        ("item_id", Type::I16),
        ("amount", Type::I16),
    ]),
    ("TradeIngredient", &[
        ("item_id", Type::I16),
        ("amount", Type::I16),
    ]),
]
