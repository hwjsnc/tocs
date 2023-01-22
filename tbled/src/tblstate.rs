/*
Tbl entries aren't necessarily sorted in any obvious or logical order, probably because these files are compiled line by line from handwritten source files.
I would be pretty surprised if the order of entries was important, but I don't know for sure, so I think it's a good idea to preserve the order wherever possible.
Since we want to view and edit the lines by header name (aka entry type, basically one table inside the tbl file), this requires a somewhat complicated data structure, which I will attempt to explain in this comment.
It is important to note that entries of the same type are not necessarily in one contiguous chunk; different entry types can alternate between each other.

Another complication is that we want to preserve edit history to enable the user to undo and redo changes.
Though the tbl files aren't too big, it's a bad idea to independently store the full state of the tbl after each edit.

Hence, we store the rows in the tbl as an immutable vector, which can look up, insert, delete, and update entries in O(log(n)).
When changing a vector, only the changed part is actually stored and the unchanged part isn't copied but only referenced.
(The real story is more complicated, but that shouldn't matter here.)

To get fast lookups for all entries with a given header name, we then store a vector of all row indices that have this header name.
These shouldn't be directly accessed and must be properly updated whenever a row is inserted or deleted.

To get undo/redo functionality, we also store two stacks of previous and later states that we update whenever a change is made.
We also store the change itself so we can provide this information to the user (e.g. show "undo (duplicate row 42 in 'magic')" in the menu).
*/

use tocs::tbl::schema::AtomicValue as Value;
use tocs::tbl::schema::Attribute;
use tocs::tbl::Header;

use std::borrow::Borrow as _;

struct TblStateWithoutHistory {
    /// All tbl rows in their original order (or the user-desired order if there have been changes)
    all_rows: im::Vector<std::rc::Rc<tocs::tbl::Entry>>, // we use refcounting because cloning a row is somewhat expensive and it needs to be done for im-Vector changes - not all entries are cloned, but up to 256 (?)
}

impl FromIterator<tocs::tbl::Entry> for TblStateWithoutHistory {
    fn from_iter<I: IntoIterator<Item = tocs::tbl::Entry>>(iter: I) -> Self {
        Self {
            all_rows: iter.into_iter().map(std::rc::Rc::new).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Change {
    DuplicateRow {
        header: Header,
        row: usize,
    },
    RemoveRow {
        header: Header,
        row: usize,
    },
    UpdateCell {
        header: Header,
        row: usize,
        column: usize,
        value: Value, // TODO: string?
    },
    ImportTable {
        header: Header,
        rows: im::Vector<std::rc::Rc<tocs::tbl::Entry>>,
    },
}

pub struct TblStateHistory {
    current: TblStateWithoutHistory,

    /// All (unique) headers with corresponding rows
    headers: indexmap::IndexSet<Header>,

    // We save the "current" header (i.e. the header of the row that was last accessed) with the expectation that many more operations on the rows with the same header will follow
    current_header: Option<Header>,
    rows_for_current_header: std::vec::Vec<usize>,

    /// undo stack
    ///
    /// Stores older states and the change that brings us to the NEXT state
    undo: std::vec::Vec<(TblStateWithoutHistory, Change)>,

    /// undo stack
    ///
    /// Stores newer states and the change that brings the PREVIOUS state to "this one" (inverse meaning compared to 'undo'!).
    redo: std::vec::Vec<(Change, TblStateWithoutHistory)>,
}

impl TblStateWithoutHistory {
    fn duplicate_row(&self, rowid: usize) -> Self {
        let mut all_rows = self.all_rows.clone(); // O(1) for immutable vector
        let row = all_rows[rowid].clone();
        all_rows.insert(rowid + 1, row);
        TblStateWithoutHistory { all_rows }
    }

    fn remove_row(&self, rowid: usize) -> Self {
        let mut all_rows = self.all_rows.clone(); // O(1) for immutable vector
        let _ = all_rows.remove(rowid);
        TblStateWithoutHistory { all_rows }
    }

    fn update_cell(&self, rowid: usize, colid: usize, value: Value) -> Self {
        let mut all_rows = self.all_rows.clone(); // O(1) for immutable vector
        let row = std::rc::Rc::make_mut(&mut all_rows[rowid]); // Clones inner data and gives us a reference to the cloned data
        let (_, oldval) = row.values.get_index_mut(colid).expect("column index should be valid");
        assert_eq!(oldval.r#type(), value.r#type());
        *oldval = value;
        TblStateWithoutHistory { all_rows }
    }
}

impl Change {
    fn header(&self) -> &Header {
        match self {
            Change::DuplicateRow { header, .. } => header,
            Change::RemoveRow { header, .. } => header,
            Change::UpdateCell { header, .. } => header,
            Change::ImportTable { header, .. } => header,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Change::DuplicateRow { header, row } => format!("duplicate row #{row} in table {header}"),
            Change::RemoveRow { header, row } => format!("remove row #{row} in table {header}"),
            Change::UpdateCell { header, row, column, value } => {
                let mut s = value.to_string();
                // we don't want to show the full string if it's very long, so we truncate it
                match s.char_indices().nth(15) {
                    None => {}
                    Some((idx, _)) => {
                        s.truncate(idx);
                        s.push_str("...");
                    }
                };
                format!("set value ({row}, {column}) in table {header} to '{}'", s)
                // TODO: attribute name?
            }
            Change::ImportTable { header, rows } => format!("import {} rows into table {}", rows.len(), header),
        }
    }
}

impl TblStateHistory {
    pub fn new(tbl: tocs::tbl::Tbl) -> Self {
        let headers = tbl.headers().chain(tbl.get_entries().iter().map(|(h, _)| h)).cloned().collect(); // TODO: Entry
        let entries = tocs::tbl::tbl_entries_legacy_convert(&tbl.into_entries()); // TODO: Entry
        let current = TblStateWithoutHistory::from_iter(entries.into_iter());
        Self {
            current,
            headers,
            current_header: None,
            rows_for_current_header: vec![],
            undo: vec![],
            redo: vec![],
        }
    }

    pub fn headers(&self) -> &indexmap::IndexSet<Header> {
        &self.headers
    }

    pub fn has_header(&self, header: &Header) -> bool {
        self.headers.contains(header)
    }

    fn set_header(&mut self, header: &Header) {
        if self.current_header.as_ref() != Some(header) {
            self.force_set_header(header.clone());
        }
    }

    fn force_set_header(&mut self, header: Header) {
        self.rows_for_current_header.clear();
        for (i, row) in self.current.all_rows.iter().enumerate() {
            if row.header == header {
                self.rows_for_current_header.push(i);
            }
        }
        self.current_header = Some(header);
    }

    pub fn get_cell_contents<'a>(&'a mut self, header: &Header, rowid: usize, colid: usize) -> Option<(&'a Attribute, &'a Value)> {
        self.set_header(header);
        let idx = self.rows_for_current_header.get(rowid).copied()?;
        let row = self.current.all_rows.get(idx)?;
        assert_eq!(&row.header, header);
        row.values.get_index(colid)
    }

    pub fn next_undo(&self) -> Option<&Change> {
        self.undo.last().map(|(_, change)| change)
    }

    pub fn next_redo(&self) -> Option<&Change> {
        self.redo.last().map(|(change, _)| change)
    }

    pub fn undo(&mut self) -> Option<()> {
        self.undo.pop().map(|(state, change)| {
            let oldcurrent = std::mem::replace(&mut self.current, state);
            self.redo.push((change, oldcurrent));
        })
    }

    pub fn redo(&mut self) -> Option<()> {
        self.redo.pop().map(|(change, state)| {
            let oldcurrent = std::mem::replace(&mut self.current, state);
            self.undo.push((oldcurrent, change));
        })
    }

    pub fn change(&mut self, change: Change) {
        let header = change.header().clone();
        self.set_header(change.header());

        let newstate = match &change {
            Change::DuplicateRow { row, .. } => self.current.duplicate_row(*row),
            Change::RemoveRow { row, .. } => self.current.remove_row(self.rows_for_current_header[*row]),
            Change::UpdateCell { row, column, value, .. } => self.current.update_cell(self.rows_for_current_header[*row], *column, value.clone()),
            Change::ImportTable { header, rows: new_rows_for_header } => {
                // There's not really a good solution here for how to order the rows when the table is split
                // We could probably use a diffing algorithm to find an approximate solution for the fewest rows assigned to an incorrect chunk.
                // The benefit doesn't nearly justify the required effort though.
                let mut got_first_row_for_header = false;
                let mut all_rows_new = im::Vector::new();
                for row in self.current.all_rows.iter() {
                    if &row.header == header {
                        if got_first_row_for_header {
                            continue;
                        } else {
                            got_first_row_for_header = true;
                            for row in new_rows_for_header {
                                all_rows_new.push_back(row.clone())
                            }
                        }
                    } else {
                        all_rows_new.push_back(row.clone())
                    }
                }
                if !got_first_row_for_header {
                    for row in new_rows_for_header {
                        all_rows_new.push_back(row.clone())
                    }
                }
                TblStateWithoutHistory { all_rows: all_rows_new }
            }
        };

        let number_of_rows_changed = match &change {
            Change::DuplicateRow { .. } => true,
            Change::RemoveRow { .. } => true,
            Change::UpdateCell { .. } => false,
            Change::ImportTable { .. } => true,
        };

        self.redo.clear();
        let oldstate = std::mem::replace(&mut self.current, newstate);
        self.undo.push((oldstate, change));

        // If row number changed, update the cache of rows for the current header!
        // It would be possible to do this a bit more efficiently, but getting it right would be difficult, so we're doing the simple thing instead.
        // This shouldn't be a huge problem unless the user adds and removes rows a lot, which seems unlikely given the current state of modding.
        if number_of_rows_changed {
            self.force_set_header(header);
        }
    }

    pub fn all_current_rows(&self) -> impl Iterator<Item = &tocs::tbl::Entry> {
        self.current.all_rows.iter().map(|rc| rc.borrow())
    }

    pub fn number_of_rows_for_header(&mut self, header: &Header) -> usize {
        self.set_header(header);
        self.rows_for_current_header.len()
    }

    pub fn rows_for_header(&mut self, header: &Header) -> impl Iterator<Item = &tocs::tbl::Entry> {
        self.set_header(header);
        self.rows_for_current_header.iter().copied().map(|i| self.current.all_rows[i].borrow())
    }

    pub fn row_for_header(&mut self, header: &Header, i: usize) -> Option<&tocs::tbl::Entry> {
        self.set_header(header);
        self.rows_for_current_header.get(i).cloned().and_then(|i| self.current.all_rows.get(i)).map(|rc| rc.borrow())
    }
}

impl Default for TblStateHistory {
    fn default() -> Self {
        Self {
            current: TblStateWithoutHistory::from_iter([]),
            headers: indexmap::IndexSet::new(),
            current_header: None,
            rows_for_current_header: vec![],
            undo: vec![],
            redo: vec![],
        }
    }
}
