// Popup window that allows selecting and reordering visible columns

use tocs::tbl::Attribute;

pub fn select_cols(_visible: &indexmap::IndexSet<Attribute>, _hidden: &indexmap::IndexSet<Attribute>) -> Option<indexmap::IndexSet<Attribute>> {
    None
    // TODO
}
