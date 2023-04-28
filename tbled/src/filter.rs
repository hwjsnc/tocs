use tocs::tbl::Attribute;

pub enum RowFilter {
    Contains {
        attribute: Option<Attribute>, // if None, any attribute can match
        substring: smol_str::SmolStr,
    },
}

pub fn row_passes_filter(entry: &tocs::tbl::Entry, filter: &RowFilter) -> bool {
    match filter {
        RowFilter::Contains { attribute, substring } => {
            if let Some(attr) = attribute {
                entry.values.get(attr).map(|v| v.to_smol_str().contains(substring.as_str())).unwrap_or(false)
            } else {
                entry.values.values().any(|v| v.to_smol_str().contains(substring.as_str()))
            }
        }
    }
}
