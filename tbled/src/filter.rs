use tocs::tbl::Attribute;

pub enum RowFilter {
    /// No filter, matches all rows
    None,
    /// Attribute contains substring, with case insensitive comparison.
    ///
    /// If attribute is None, any attribute can match
    CaseInsensitiveContains { attribute: Option<Attribute>, substring: regex::Regex },
    /// Attribute value (converted to string) equals value. Matching is case sensitive.
    Equals { attribute: Attribute, string: smol_str::SmolStr },
}

impl std::str::FromStr for RowFilter {
    type Err = (); // !

    fn from_str(s: &str) -> Result<RowFilter, Self::Err> {
        // attr=___ -> check attribute equality
        // attr:___ -> check attribute contains
        // anything else: check any attribute contains
        if s.is_empty() {
            Ok(RowFilter::None)
        } else {
            // find the first occurance of ':' or '='
            match s.chars().enumerate().find(|(_, c)| ['=', ':'].contains(c)) {
                None => Ok(RowFilter::CaseInsensitiveContains {
                    attribute: None,
                    substring: regex::RegexBuilder::new(&regex::escape(&s)).case_insensitive(true).build().expect("escaped regex should be valid"),
                }),
                Some((i, ':')) => Ok(RowFilter::CaseInsensitiveContains {
                    attribute: Some(Attribute::from(&s[..i])),
                    substring: regex::RegexBuilder::new(&regex::escape(&s[i + 1..]))
                        .case_insensitive(true)
                        .build()
                        .expect("escaped regex should be valid"),
                }),
                Some((i, '=')) => Ok(RowFilter::Equals {
                    attribute: Attribute::from(&s[..i]),
                    string: smol_str::SmolStr::from(&s[i + 1..]),
                }),
                _ => unreachable!(),
            }
        }
    }
}

pub fn row_passes_filter(entry: &tocs::tbl::Entry, filter: &RowFilter) -> bool {
    match filter {
        RowFilter::None => true,
        RowFilter::CaseInsensitiveContains { attribute, substring } => {
            if let Some(attr) = attribute {
                entry.values.get(attr).map(|v| substring.is_match(v.to_smol_str().as_str())).unwrap_or(false)
            } else {
                entry.values.values().any(|v| substring.is_match(v.to_smol_str().as_str()))
            }
        }
        RowFilter::Equals { attribute, string } => entry.values.get(attribute).map(|v| &v.to_smol_str() == string).unwrap_or(false),
    }
}
