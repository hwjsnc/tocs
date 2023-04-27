// table widget with (possibly) filtered rows and (possibly) reordered/hidden columns

use tocs::tbl::AtomicSchema as Schema;
use tocs::tbl::{Attribute, Entry, Header};

use smol_str::SmolStr;
use std::borrow::Borrow;
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
        table.set_col_width_all(40);

        table.draw_cell({
            // We move a reference to the table data into the following closure
            let table_data = table_data.clone();
            move |table, ctx, row, col, x, y, w, h| {
                let nrows = table.rows();
                let ncols = table.cols();
                log::trace!("table draw: ctx={ctx:?} row={row}/{nrows} row={col}/{ncols}, x={x}, y={y}, w={w}, h={h}");
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
                        let Some(s) = cells.get(row_idx, col_idx) else { return };
                        // TODO: draw cell
                        // to consider:
                        // - is it selected?
                        // - even or odd row?
                        // - alignment? (attribute type! -> get this from the schema)
                        fltk::draw::push_clip(x, y, w, h);
                        fltk::draw::set_draw_color(fltk::enums::Color::from_u32(0x00_ff_ff_ff));
                        fltk::draw::draw_rectf(x, y, w, h);
                        fltk::draw::set_draw_color(fltk::enums::Color::Gray0);
                        fltk::draw::draw_rect(x, y, w, h);

                        let (x, w) = if w > 5 { (x + 2, w - 4) } else { (x, w) };
                        let (y, h) = if h > 5 { (y + 2, h - 4) } else { (y, h) };
                        fltk::draw::draw_text2(&s, x, y, w, h, fltk::enums::Align::Right);
                        fltk::draw::pop_clip();
                    }
                    _ => {}
                }
            }
        });

        Self { widget: table, data: table_data }
    }

    pub fn update_data_alternative(&mut self, state: &crate::AppData) {
        if let Some(header) = state.active_entry_type.as_ref() {
            let schema = crate::schema_for_header(header, state.tbl_version.schemas());
            let attributes = schema.keys().cloned().into_iter().collect();
            let filters = &[];
            let all_tbl_rows = state.tbl_history.all_current_rows();
            self.update_data(header.clone(), &schema, &attributes, filters, all_tbl_rows)
        } else {
            let header = Header::new("").expect("empty string is valid ASCII");
            let schema = Schema::new();
            let attributes = indexmap::IndexSet::new();
            let filters = &[];
            let all_tbl_rows = &[];
            self.update_data(header, &schema, &attributes, filters, all_tbl_rows.into_iter());
        }
        self.auto_size_cells(); // TODO: only when necessary
    }

    pub fn update_data<I, E>(&mut self, header_name: Header, schema: &Schema, attributes: &indexmap::IndexSet<Attribute>, row_filters: &[RowFilter], all_tbl_rows: I)
    where
        I: Iterator<Item = E>,
        E: Borrow<Entry>,
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
            if row.borrow().header == header_name && row_filters.iter().all(|filter| row_passes_filter(row.borrow(), filter)) {
                for (attr, attr_idx) in columns_and_original_indexes.iter() {
                    let (k, v) = row.borrow().values.get_index(*attr_idx).expect("entry attributes should correspond to schema");
                    assert_eq!(k, attr, "entry should have attributes in the same order as the schema");
                    row_major_vector_of_cells.push(v.to_smol_str());
                }
            }
        }

        // Now we create the new Grid and swap it in for the empty placeholder Grid
        let mut new_cells = grid::Grid::from_vec(row_major_vector_of_cells, attributes.len());
        std::mem::swap(cells, &mut new_cells);

        // We need to drop the guard before redrawing the table to allow draw_cell access to the data
        let (nrows, ncols) = (cells.rows().try_into().unwrap(), cells.cols().try_into().unwrap());
        drop(guard);

        self.widget.set_rows(nrows);
        self.widget.set_cols(ncols);
        log::debug!("table data updated; {} rows, {} columns", self.widget.rows(), self.widget.cols());
        self.widget.redraw();
    }

    pub fn auto_size_cells(&mut self) {
        let guard = self.data.lock().unwrap();
        let TableData {
            columns_and_original_indexes, cells, ..
        } = &*guard;
        let mut column_widths: std::vec::Vec<i32> = columns_and_original_indexes.iter().map(|(attr, _)| fltk::draw::measure(attr, false).0).collect();
        let nrows: i32 = cells.rows().try_into().unwrap();
        let mut row_height = 1;
        for r in 0..cells.rows() {
            for c in 0..cells.cols() {
                let (w, h) = fltk::draw::measure(cells.get(r, c).unwrap(), false);
                column_widths[c] = std::cmp::max(column_widths[c], w);
                row_height = std::cmp::max(row_height, h);
            }
        }
        // We need to drop the guard before redrawing the table to allow draw_cell access to the data
        drop(guard);

        log::trace!("column widths: {:?} rows: {} row height: {}", column_widths, nrows, row_height);

        for (i, w) in column_widths.into_iter().enumerate() {
            self.widget.set_col_width(i.try_into().unwrap(), w);
        }
        self.widget.set_row_height_all(row_height);

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
