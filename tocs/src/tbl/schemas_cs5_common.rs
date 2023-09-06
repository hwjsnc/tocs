&[
    ("dlc_item", &[
        ("id", Type::U16),
        ("count", Type::U16),
    ]),
    ("effect", &[
        ("id", Type::U16),
        ("data", Type::Repeat(3, Box::new(Type::I32))),
    ]),
    ("item_drop", &[
        ("item_id", Type::U16),
        ("amount_min", Type::U16),
        ("amount_max", Type::U16),
        ("chance", Type::U8),
    ]),
    ("QSCook_ingredient", &[
        ("id", Type::U16),
        ("count", Type::U16),
    ]),
    ("SCraftLvBox_upgrade", &[
        ("character", Type::I16),
        ("base_craft", Type::I16),
        ("upgraded_craft", Type::I16),
    ]),
    ("TradeIngredient", &[
        ("item_id", Type::I16),
        ("amount", Type::I16),
    ]),
]
