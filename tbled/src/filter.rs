pub enum RowFilter {}

pub fn row_passes_filter(_entry: &tocs::tbl::Entry, _filter: &RowFilter) -> bool {
    return true;
}
