// table widget with (possibly) filtered rows and (possibly) reordered/hidden columns

use tocs::tbl::AtomicSchema as Schema;
use tocs::tbl::{Attribute, Entry, Header};

use smol_str::SmolStr;
use std::sync::{Arc, Mutex};

use fltk::prelude::TableExt as _;
use fltk::prelude::WidgetExt as _;

pub enum RowFilter {
    // TODO: 'any cell includes "Rean"', 'attribute "foo" = 4', stuff like that
}

fn row_passes_filter(_entry: &Entry, _filter: &RowFilter) -> bool {
    return true; // for now
}

struct TableData {
    columns_and_original_indexes: std::vec::Vec<(Attribute, usize)>,
    original_row_indexes: std::vec::Vec<usize>,
    cells: grid::Grid<SmolStr>,
}

pub struct FilteredTable {
    widget: fltk::table::Table,
    data: Arc<Mutex<TableData>>,
}

impl FilteredTable {
    pub fn new() -> Self {
        let table_data = Arc::new(Mutex::new(TableData {
            columns_and_original_indexes: std::vec::Vec::new(),
            original_row_indexes: std::vec::Vec::new(),
            cells: grid::Grid::new(0, 0),
        }));

        let mut table = fltk::table::Table::default();
        table.set_col_header(true);
        table.set_col_resize(true);
        table.set_row_header(true);
        table.set_row_resize(true);

        table.draw_cell({
            // We move a reference to the table data into the following closure
            let table_data = table_data.clone();
            move |table, ctx, row, col, x, y, w, h| {
                log::trace!("table draw {row} {col}, {x}, {y}, {w}, {h}");
                let mut guard = table_data.lock().unwrap();
                let TableData {
                    columns_and_original_indexes,
                    original_row_indexes: _,
                    cells,
                } = &mut *guard;
                let Ok(row_idx) = usize::try_from(row) else { return };
                let Ok(col_idx) = usize::try_from(col) else { return };
                match ctx {
                    fltk::table::TableContext::ColHeader => {
                        let Some((attr, _)) = columns_and_original_indexes.get(col_idx) else { return };
                        crate::draw_header(&attr, x, y, w, h); // Column titles
                    }
                    fltk::table::TableContext::RowHeader => {
                        crate::draw_header("", x, y, w, h); // Row titles
                    }
                    fltk::table::TableContext::Cell => {
                        // TODO: draw cell
                        // to consider:
                        // - is it selected?
                        // - even or odd row?
                        // - alignment? (attribute type! -> get this from the schema)
                    }
                    _ => (),
                }
            }
        });

        Self {
            widget: fltk::table::Table::default(),
            data: table_data,
        }
    }

    pub fn update_data<I, E>(&mut self, schema: &Schema, header_name: Header, attributes: indexmap::IndexSet<Attribute>, row_filters: &[RowFilter], all_tbl_rows: I)
    where
        I: Iterator<Item = E>,
        E: AsRef<Entry>,
    {
        let mut guard = self.data.lock().unwrap();
        let TableData {
            ref mut columns_and_original_indexes,
            ref mut original_row_indexes,
            ref mut cells,
        } = &mut *guard;

        // The Grid datastructure doesn't have a resize operation and we don't want to allocate a new one.
        // Instead we clear the grid (which clears the underlying vector but keeps its capacity), then create a new grid from that old vector.
        // This allows the new grid (with possibly different column count!) to grow without allocations at least until it reaches the size of the old grid.
        // However, Rust doesn't allow us to move self.grid, so we have to instead create a new empty Grid (this should not allocate), and then swap the two
        cells.clear();
        let mut new_cells = grid::Grid::new(0, 0);
        std::mem::swap(cells, &mut new_cells);
        let mut row_major_vector_of_cells = new_cells.into_vec();

        // update columns
        columns_and_original_indexes.clear();
        for attr in attributes.iter() {
            let original_index = schema.get_index_of(attr).expect("attribute not found in schema");
            columns_and_original_indexes.push((attr.clone(), original_index));
        }

        // update rows
        original_row_indexes.clear();
        for row in all_tbl_rows {
            if row.as_ref().header == header_name && row_filters.iter().all(|filter| row_passes_filter(row.as_ref(), filter)) {
                for (attr, attr_idx) in columns_and_original_indexes.iter() {
                    let (k, v) = row.as_ref().values.get_index(*attr_idx).expect("entry attributes should correspond to schema");
                    assert_eq!(k, attr, "entry should have attributes in the same order as the schema");
                    row_major_vector_of_cells.push(v.to_smol_str());
                }
            }
        }

        // Now we create the new Grid and swap it in for the empty placeholder Grid
        let mut new_cells = grid::Grid::from_vec(row_major_vector_of_cells, attributes.len());
        std::mem::swap(cells, &mut new_cells);

        self.widget.set_cols(i32::try_from(cells.cols()).unwrap());
        self.widget.set_rows(i32::try_from(cells.rows()).unwrap());

        // We need to drop the guard before redrawing the table to allow draw_cell access to the data
        drop(guard);
        self.widget.redraw();
    }

    pub fn emit<T: 'static + Clone + Send + Sync>(&mut self, sender: fltk::app::Sender<T>, msg: T) {
        self.widget.emit(sender, msg)
    }

    fn original_row(&self, filtered_row_index: i32) -> usize {
        let row: usize = filtered_row_index.try_into().unwrap();
        let guard = self.data.lock().unwrap();
        let TableData { ref original_row_indexes, .. } = &*guard;
        original_row_indexes[row]
    }

    fn original_col(&self, filtered_col_index: i32) -> usize {
        let col: usize = filtered_col_index.try_into().unwrap();
        let guard = self.data.lock().unwrap();
        let TableData { ref columns_and_original_indexes, .. } = &*guard;
        columns_and_original_indexes[col].1
    }

    pub fn selected_cell(&self) -> Option<(usize, usize)> {
        let (row, col, row2, col2) = self.widget.try_get_selection()?;
        if row == row2 && col == col2 {
            Some((self.original_row(row), self.original_col(col)))
        } else {
            None
        }
    }

    pub fn selected_row(&self) -> Option<usize> {
        let (row, col, row2, col2) = self.widget.try_get_selection()?;
        if row == row2 && col == 0 && col2 == self.widget.cols() - 1 {
            Some(self.original_row(row))
        } else {
            None
        }
    }
}
