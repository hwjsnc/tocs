use tocs::tbl::Attribute;

pub enum RowFilter {
    CaseInsensitiveContains {
        attribute: Option<Attribute>, // if None, any attribute can match
        substring: regex::Regex,
    },
}

impl RowFilter {
    pub fn case_insensitive_contains(attribute: Option<Attribute>, s: smol_str::SmolStr) -> Self {
        RowFilter::CaseInsensitiveContains {
            attribute,
            substring: regex::RegexBuilder::new(&regex::escape(&s)).case_insensitive(true).build().expect("escaped regex should be valid"),
        }
    }
}

pub fn row_passes_filter(entry: &tocs::tbl::Entry, filter: &RowFilter) -> bool {
    match filter {
        RowFilter::CaseInsensitiveContains { attribute, substring } => {
            if let Some(attr) = attribute {
                entry.values.get(attr).map(|v| substring.is_match(v.to_smol_str().as_str())).unwrap_or(false)
            } else {
                entry.values.values().any(|v| substring.is_match(v.to_smol_str().as_str()))
            }
        }
    }
}
