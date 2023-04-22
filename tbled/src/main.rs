#![windows_subsystem = "windows"] // don't open terminal on windows

mod tblstate;
mod warn_diag;

use fltk::prelude::GroupExt as _;
use fltk::prelude::InputExt as _;
use fltk::prelude::MenuExt as _;
use fltk::prelude::TableExt as _;
use fltk::prelude::WidgetExt as _;

use clap::Parser;

use indexmap::IndexMap;

use tocs::tbl::AtomicAttributeValues as AttributeValues;
use tocs::tbl::AtomicSchema as Schema;
use tocs::tbl::Header;

use tblstate::{Change, TblStateHistory};

#[derive(Debug, Parser)]
struct CliArgs {
    #[arg(short, long)]
    game_version: Option<tocs::tbl::Version<'static>>,
    #[arg(short, long)]
    file: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug)]
enum Msg {
    SelectAndLoadFile(tocs::tbl::Version<'static>), // user wants to load a file
    SaveFile,                                       // user wants to save to the same file
    SelectAndSaveFile,                              // user wants to save a file
    ExportCSV,                                      // user wants to export file to csv
    ImportCSV,                                      // user wants to import table from csv
    ShowWarnings,                                   // user wants to see warnings
    FileReloaded,                                   // user loaded a different tbl file and we have to update all widgets accordingly
    UpdateTable,                                    // user selected a different header; update the table widget
    Undo,                                           // user wants to undo last action
    Redo,                                           // user wants to redo last action
    DuplicateEntry,                                 // user wants to duplicate the current row
    DeleteEntry,                                    // user wants to delete the current row
    TableEvent,                                     // an arbitrary event on the table
    DoneEditing,                                    // user is done editing, now the value should be updated
}

struct Widgets {
    menu: fltk::menu::SysMenuBar,
    input: fltk::input::Input,
    header_selector: fltk::misc::InputChoice,
    table: fltk::table::Table,
}

pub struct AppData {
    tbl_history: TblStateHistory,
    tbl_version: tocs::tbl::Version<'static>,
    path: std::path::PathBuf,
    active_entry_type: Option<Header>,
    warnings: std::vec::Vec<tocs::tbl::DeserializeError>,
    //edited: std::collections::HashSet<(i32, i32)>, // edited since last save
}

struct App {
    app: fltk::app::App,
    receiver: fltk::app::Receiver<Msg>,
    widgets: Widgets,
    state: std::sync::Arc<std::sync::Mutex<AppData>>,
}

impl App {
    pub fn new(state: AppData) -> Self {
        let state = std::sync::Arc::new(std::sync::Mutex::new(state));

        fltk::draw::set_font(fltk::enums::Font::Helvetica, 14);

        let (w, h) = (1280, 720);
        let app = fltk::app::App::default();
        let (sender, receiver) = fltk::app::channel();
        let mut win = fltk::window::Window::default()
            .with_size(w, h)
            .with_label(&format!("{} version {}", env! {"CARGO_PKG_NAME"}, env! {"CARGO_PKG_VERSION"}));
        win.set_callback(|_| {
            // override close window on pressing escape
            if fltk::app::event() == fltk::enums::Event::Close {
                fltk::app::quit();
            }
        });
        let mut flex = fltk::group::Flex::default().size_of_parent();
        flex.set_type(fltk::group::FlexType::Column);

        let mut flex_row = fltk::group::Flex::default().size_of_parent();
        flex_row.set_type(fltk::group::FlexType::Row);

        let mut menu = fltk::menu::SysMenuBar::default();
        menu.add_emit(
            "&Load/Load CS1 tbl (XSeed PC version, English)\t",
            fltk::enums::Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::SelectAndLoadFile(tocs::tbl::Version::CS1En),
        );

        menu.add_emit(
            "&Load/Load CS2 tbl (XSeed PC version, English)\t",
            fltk::enums::Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::SelectAndLoadFile(tocs::tbl::Version::CS2En),
        );

        menu.add_emit(
            "&Load/Load CS3 tbl (NISA PC version)\t",
            fltk::enums::Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::SelectAndLoadFile(tocs::tbl::Version::CS3),
        );

        menu.add_emit(
            "&Load/Load CS4 tbl (NISA Switch version)\t",
            fltk::enums::Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::SelectAndLoadFile(tocs::tbl::Version::CS4),
        );

        menu.add_emit("&Save/Save\t", fltk::enums::Shortcut::Ctrl | 's', fltk::menu::MenuFlag::Normal, sender.clone(), Msg::SaveFile);

        menu.add_emit(
            "&Save/Save as...\t",
            fltk::enums::Shortcut::Ctrl | fltk::enums::Shortcut::Shift | 's',
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::SelectAndSaveFile,
        );

        menu.add_emit(
            "&Warnings/Show warnings...\t",
            fltk::enums::Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::ShowWarnings,
        );

        menu.add_emit("&Export/Export to csv...\t", fltk::enums::Shortcut::None, fltk::menu::MenuFlag::Normal, sender.clone(), Msg::ExportCSV);

        menu.add_emit(
            "&Import/Import from csv...\t",
            fltk::enums::Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::ImportCSV,
        );

        menu.add_emit("&Edit/Undo\t", fltk::enums::Shortcut::Ctrl | 'z', fltk::menu::MenuFlag::Inactive, sender.clone(), Msg::Undo);

        menu.add_emit(
            "&Edit/Redo\t",
            fltk::enums::Shortcut::Ctrl | fltk::enums::Shortcut::Shift | 'z',
            fltk::menu::MenuFlag::Inactive,
            sender.clone(),
            Msg::Redo,
        );

        menu.add_emit(
            "&Edit/Duplicate currently selected row\t",
            fltk::enums::Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::DuplicateEntry,
        );

        menu.add_emit(
            "&Edit/Delete currently selected row\t",
            fltk::enums::Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            sender.clone(),
            Msg::DeleteEntry,
        );

        let mut header_selector = fltk::misc::InputChoice::default();
        header_selector.emit(sender.clone(), Msg::UpdateTable);
        header_selector.input().set_readonly(true);
        flex_row.set_size(&header_selector, 200);

        flex_row.end();
        flex.set_size(&flex_row, 26);
        let mut flex_row = fltk::group::Flex::default().size_of_parent();
        flex_row.set_type(fltk::group::FlexType::Row);

        let mut input = fltk::input::Input::default();
        input.set_trigger(fltk::enums::CallbackTrigger::EnterKeyAlways);
        input.emit(sender.clone(), Msg::DoneEditing);

        flex_row.end();
        flex.set_size(&flex_row, 26);

        let mut table = fltk::table::Table::default();
        table.set_col_header(true);
        table.set_col_width_all(40);
        table.set_col_resize(true);
        table.set_row_header(true);
        table.set_row_resize(true);

        table.emit(sender, Msg::TableEvent);

        let state_clone = state.clone();
        table.draw_cell(move |table, ctx, row, col, x, y, w, h| {
            log::trace!("table draw");
            // called when the table is drawn (both the first time and due to events)
            let mut state = state_clone.lock().unwrap();
            let AppData { tbl_history, active_entry_type, .. } = &mut *state;
            let header = if let Some(hdr) = active_entry_type.as_ref() {
                hdr
            } else {
                return;
            };
            let row_idx: usize = if let Ok(row_idx) = row.try_into() {
                row_idx
            } else {
                return;
            };
            let col_idx: usize = if let Ok(col_idx) = col.try_into() {
                col_idx
            } else {
                return;
            };
            let (attribute, value) = if let Some(tup) = tbl_history.get_cell_contents(header, row_idx, col_idx) {
                tup
            } else {
                return;
            };
            match ctx {
                fltk::table::TableContext::ColHeader => {
                    draw_header(attribute, x, y, w, h); // Column titles
                }
                fltk::table::TableContext::RowHeader => {
                    draw_header("", x, y, w, h); // Row titles
                }
                fltk::table::TableContext::Cell => {
                    let celltype = if table.is_selected(row, col) {
                        CellType::Selected
                    } else if row % 2 == 0 {
                        CellType::EvenRow
                    } else {
                        CellType::OddRow
                    };
                    draw_data(value, x, y, w, h, celltype);
                }
                _ => (),
            }
        });

        flex.end();

        win.make_resizable(true);
        win.end();
        win.show();

        let mut this = Self {
            app,
            receiver,
            widgets: Widgets { menu, header_selector, input, table },
            state,
        };

        this.update(&Some(Msg::FileReloaded));

        this
    }

    pub fn run(mut self) {
        while self.app.wait() {
            let msg = self.receiver.recv();
            if msg.is_some() {
                self.update(&msg);
            }
        }
    }

    pub fn update(&mut self, msg: &Option<Msg>) {
        log::debug!("update: {msg:?}");
        match msg {
            Some(Msg::SelectAndLoadFile(version)) => {
                // open a file browser dialog to select the file
                let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseFile);
                diag.set_title("Select .tbl file to load ...");
                diag.show();
                let new_path = diag.filename();
                if new_path.as_os_str().is_empty() {
                    return;
                }
                {
                    // reset state related to tbl data
                    let mut state = self.state.lock().unwrap();
                    let AppData {
                        tbl_history,
                        active_entry_type,
                        tbl_version,
                        path,
                        warnings,
                    } = &mut *state;
                    *active_entry_type = None;
                    *tbl_version = *version;
                    warnings.clear();
                    // try loading data from file
                    match tocs::tbl::read_file(&new_path, *version) {
                        tocs::tbl::ReadTblResult::Ok(tbl) => {
                            *tbl_history = TblStateHistory::new(tbl);
                            *path = new_path;
                        }
                        tocs::tbl::ReadTblResult::Warn { tbl, warnings: w } => {
                            *warnings = w;
                            *tbl_history = TblStateHistory::new(tbl);
                            *path = new_path;
                            // if errors are harmless (e.g. all length bytes & all headers are in the schema), don't alert
                            let schemas = tbl_version.schemas();
                            if tbl_history.all_current_rows().all(|entry| schemas.contains_key(&entry.header))
                                && warnings.iter().all(|w| matches!(w.detail, tocs::tbl::DeserializeErrorDetail::EntryDataLengthMismatch { .. }))
                            {
                                // it's fine
                            } else {
                                fltk::dialog::alert_default(
                                    "Warning: Some errors/inconsistencies in the file were detected. Please look at them under the Warnings menu option and ensure that they are harmless",
                                );
                            }
                        }
                        tocs::tbl::ReadTblResult::Err(e) => {
                            fltk::dialog::alert_default(&format!("error loading file: {}", e));
                            *path = std::path::PathBuf::new();
                        }
                    }
                }
                // we need to update other ui elements
                self.update(&Some(Msg::FileReloaded));
            }
            Some(Msg::SaveFile) => {
                // try saving data to file
                let state = self.state.lock().unwrap();
                let tbl = tocs::tbl::Tbl::new(state.tbl_version, state.tbl_history.headers().iter().cloned(), state.tbl_history.all_current_rows().cloned());
                let result = tocs::tbl::write_file(&state.path, &tbl);
                match result {
                    Ok(_) => {
                        // TODO: clear changed cell flags
                    }
                    Err(e) => {
                        fltk::dialog::alert_default(&format!("error saving file: {:?}", e));
                    }
                }
            }
            Some(Msg::SelectAndSaveFile) => {
                // open a file browser dialog to select the file
                let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseSaveFile);
                diag.set_title("Select .tbl file to save to ...");
                diag.show();
                let path = diag.filename();
                if path.as_os_str().is_empty() {
                    return;
                }
                // try saving data to file
                let state = self.state.lock().unwrap();
                let tbl = tocs::tbl::Tbl::new(state.tbl_version, state.tbl_history.headers().iter().cloned(), state.tbl_history.all_current_rows().cloned());
                let result = tocs::tbl::write_file(&path, &tbl);
                match result {
                    Ok(_) => {
                        // TODO: clear changed cell flags
                    }
                    Err(e) => {
                        fltk::dialog::alert_default(&format!("error saving file: {:?}", e));
                    }
                }
            }
            Some(Msg::ExportCSV) => {
                let mut state = self.state.lock().unwrap();
                let AppData {
                    tbl_history,
                    tbl_version,
                    active_entry_type,
                    ..
                } = &mut *state;
                if let Some(hdr) = active_entry_type.as_ref() {
                    // open a file browser dialog to select the file
                    let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseSaveFile);
                    diag.set_title("Select .tbl file to save to ...");
                    diag.set_preset_file(&format!("{}.csv", hdr));
                    diag.show();
                    let path = diag.filename();
                    if path.as_os_str().is_empty() {
                        return;
                    }
                    let entries: std::vec::Vec<_> = tbl_history.rows_for_header(hdr).map(|entry| &entry.values).cloned().collect();
                    match export_csv(&path, schema_for_header(hdr, tbl_version.schemas()), &entries) {
                        Ok(_) => {}
                        Err(e) => {
                            fltk::dialog::alert_default(&format!("error exporting to csv: {:?}", e));
                        }
                    }
                } else {
                    fltk::dialog::alert_default("no entry type selected, can't export");
                }
            }
            Some(Msg::ImportCSV) => {
                let mut state = self.state.lock().unwrap();
                if let Some(hdr) = state.active_entry_type.as_ref().cloned() {
                    // open a file browser dialog to select the file
                    let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseFile);
                    diag.set_title("Select .csv file to import...");
                    //diag.set_preset_file(&format!("{}.csv", hdr)); // freezes if this file doesn't exist
                    diag.show();
                    let path = diag.filename();
                    if path.as_os_str().is_empty() {
                        return;
                    }
                    match import_csv(&path, schema_for_header(&hdr, state.tbl_version.schemas())) {
                        Ok(entries) => {
                            let rows = entries.into_iter().map(|values| std::rc::Rc::new(tocs::tbl::Entry { header: hdr.clone(), values })).collect();
                            state.tbl_history.change(Change::ImportTable { header: hdr.clone(), rows });
                        }
                        Err(e) => {
                            fltk::dialog::alert_default(&format!("error importing data from csv: {:?}", e));
                        }
                    }
                } else {
                    fltk::dialog::alert_default("no entry type selected, can't import");
                }
                drop(state);
                // Note: mutex must be free for this function because it triggers table redraws:
                self.update(&Some(Msg::UpdateTable));
            }
            Some(Msg::FileReloaded) => {
                {
                    let mut state = self.state.lock().unwrap();
                    let AppData {
                        active_entry_type,
                        tbl_version: _,
                        path: _,
                        tbl_history,
                        warnings,
                    } = &mut *state;
                    // if there are warnings, show button to display them
                    if warnings.is_empty() {
                        self.widgets.menu.find_item("&Warnings/Show warnings...\t").unwrap().deactivate();
                    } else {
                        self.widgets.menu.find_item("&Warnings/Show warnings...\t").unwrap().activate();
                    }
                    // reset header selector
                    *active_entry_type = self.widgets.header_selector.value().and_then(|hdr| {
                        let hdr = Header::new(&hdr).expect("it shouldn't be possible to set the selector to an invalid value");
                        if tbl_history.has_header(&hdr) {
                            Some(hdr)
                        } else {
                            None
                        }
                    });
                    self.widgets.header_selector.clear();
                    for hdr in tbl_history.headers() {
                        self.widgets.header_selector.add(hdr.as_str());
                    }
                    if let Some(hdr) = tbl_history.headers().iter().next() {
                        self.widgets.header_selector.set_value(hdr.as_str());
                        state.active_entry_type = Some(hdr.clone());
                    }
                }
                // Note: mutex must be free for this function because it triggers table redraws:
                self.update(&Some(Msg::UpdateTable));
            }
            Some(Msg::UpdateTable) => {
                let mut state = self.state.lock().unwrap();
                let AppData {
                    active_entry_type,
                    tbl_history,
                    tbl_version,
                    ..
                } = &mut *state;
                // reset header selector
                *active_entry_type = self.widgets.header_selector.value().and_then(|hdr| {
                    let hdr = Header::new(&hdr).expect("it shouldn't be possible to set the selector to an invalid value");
                    if tbl_history.has_header(&hdr) {
                        Some(hdr)
                    } else {
                        None
                    }
                });
                if let Some(hdr) = active_entry_type {
                    log::trace!("beep");
                    // width of each column = max(header width, width of each entry in that column)
                    // width of each row = max(width of all entries in all rows)
                    let attrs = schema_for_header(hdr, tbl_version.schemas());
                    let mut column_widths: std::vec::Vec<i32> = attrs.iter().map(|(hdr, _)| fltk::draw::measure(hdr, false).0).collect();
                    let nrows: i32 = tbl_history.number_of_rows_for_header(hdr).try_into().unwrap();
                    let mut row_height = 1;
                    for row in tbl_history.rows_for_header(hdr) {
                        for (i, (_, val)) in row.values.iter().enumerate() {
                            let (w, h) = fltk::draw::measure(&val.to_string(), false);
                            column_widths[i] = std::cmp::max(column_widths[i], w);
                            row_height = std::cmp::max(row_height, h);
                        }
                    }
                    log::trace!("{} {} {}", column_widths.len(), nrows, row_height);
                    drop(state);
                    // Note: mutex must be free for this function because it triggers table redraws:
                    self.update_table(&column_widths, nrows, row_height);
                    self.update_undo_redo_menu();
                }
            }
            Some(Msg::DuplicateEntry) => {
                if let Some(row) = try_get_selected_row(&self.widgets.table) {
                    let row: usize = row.try_into().unwrap();
                    let mut state = self.state.lock().unwrap();
                    let AppData { active_entry_type, tbl_history, .. } = &mut *state;
                    if let Some(hdr) = active_entry_type {
                        tbl_history.change(Change::DuplicateRow { header: hdr.clone(), row })
                    }
                    drop(state);
                    // Note: mutex must be free for this function because it triggers table redraws:
                    self.update(&Some(Msg::UpdateTable));
                } else {
                    fltk::dialog::alert_default("please select exactly one row");
                    // TODO: allow multiple? spliced or blockwise?
                }
            }
            Some(Msg::DeleteEntry) => {
                if let Some(row) = try_get_selected_row(&self.widgets.table) {
                    let row: usize = row.try_into().unwrap();
                    let mut state = self.state.lock().unwrap();
                    let AppData { active_entry_type, tbl_history, .. } = &mut *state;
                    if let Some(hdr) = active_entry_type {
                        tbl_history.change(Change::RemoveRow { header: hdr.clone(), row })
                    }
                    drop(state);
                    // Note: mutex must be free for this function because it triggers table redraws:
                    self.update(&Some(Msg::UpdateTable));
                } else {
                    fltk::dialog::alert_default("please select exactly one row");
                    // TODO: allow multiple
                }
            }
            Some(Msg::Undo) => {
                let mut state = self.state.lock().unwrap();
                state.tbl_history.undo();
                drop(state);
                // Note: mutex must be free for this function because it triggers table redraws:
                self.update(&Some(Msg::UpdateTable));
            }
            Some(Msg::Redo) => {
                let mut state = self.state.lock().unwrap();
                state.tbl_history.redo();
                drop(state);
                // Note: mutex must be free for this function because it triggers table redraws:
                self.update(&Some(Msg::UpdateTable));
            }
            Some(Msg::TableEvent) => {
                match fltk::app::event() {
                    fltk::enums::Event::Released if fltk::app::event_clicks() => {
                        // find currently selected cell (if it's exactly one)
                        let table = &mut self.widgets.table;
                        log::debug!("double click on table");
                        if let Some((row, col)) = try_get_selected_cell(table) {
                            // copy table value to input
                            log::debug!("double click on cell");
                            let mut state = self.state.lock().unwrap();
                            self.widgets.input.set_value(&get_cell_value(&mut state, row, col).unwrap().to_string());
                            self.widgets.input.take_focus().unwrap();
                            self.widgets.input.redraw();
                        }
                    }
                    _ => {}
                }
            }
            Some(Msg::DoneEditing) => {
                // find the currently selected cell (if it's exactly one)
                let input = &mut self.widgets.input;
                let table = &mut self.widgets.table;
                if let Some((row, col)) = try_get_selected_cell(table) {
                    // copy text input to selected cell
                    {
                        let row: usize = row.try_into().unwrap();
                        let column: usize = col.try_into().unwrap();
                        let mut state = self.state.lock().unwrap();
                        let text = input.value();
                        let header = state.active_entry_type.clone().unwrap();
                        let ty = schema_for_header(&header, state.tbl_version.schemas()).get_index(column).unwrap().1;
                        if let Ok(value) = tocs::tbl::AtomicValue::from_str_nonstrict(ty, &text) {
                            state.tbl_history.change(Change::UpdateCell { header, row, column, value });
                        } else {
                            fltk::dialog::alert_default(&format!("invalid input for cell, expected a {}", ty.describe()));
                        }
                    }
                    table.redraw();
                }
                self.update_undo_redo_menu();
            }
            Some(Msg::ShowWarnings) => {
                let mut state = self.state.lock().unwrap();
                let AppData { warnings, .. } = &mut *state;
                warn_diag::WarningDialog::new(warnings.iter());
            }
            None => {}
        }
    }

    fn update_table(&mut self, column_widths: &[i32], nrows: i32, row_height: i32) {
        let ncols = column_widths.len().try_into().unwrap_or(i32::MAX);
        let table = &mut self.widgets.table;
        table.set_rows(nrows);
        table.set_cols(ncols);
        for (w, i) in column_widths.iter().zip(0..ncols) {
            table.set_col_width(i, w + 4);
        }
        table.set_row_height_all(row_height + 4);
    }

    fn update_undo_redo_menu(&mut self) {
        let edit_menu = self.widgets.menu.find_item("&Edit").unwrap();
        assert!(edit_menu.is_submenu());
        let mut undo_item = edit_menu.at(1).unwrap();
        let mut redo_item = edit_menu.at(2).unwrap();
        assert!(undo_item.label().unwrap().starts_with("Undo"));
        assert!(redo_item.label().unwrap().starts_with("Redo"));

        let state = self.state.lock().unwrap();

        if let Some(change) = state.tbl_history.next_undo() {
            undo_item.set_label(&format!("Undo: {}\t", change.describe()));
            undo_item.activate();
        } else {
            undo_item.set_label("Undo\t");
            undo_item.deactivate();
        }

        if let Some(change) = state.tbl_history.next_redo() {
            redo_item.set_label(&format!("Redo: {}\t", change.describe()));
            redo_item.activate();
        } else {
            redo_item.set_label("Redo\t");
            redo_item.deactivate();
        }
    }
}

fn get_cell_value(state: &mut AppData, row: i32, col: i32) -> Option<&tocs::tbl::AtomicValue> {
    let row: usize = row.try_into().unwrap();
    let col: usize = col.try_into().unwrap();
    let AppData { active_entry_type, tbl_history, .. } = state;

    active_entry_type
        .as_ref()
        .and_then(|hdr| tbl_history.row_for_header(hdr, row))
        .and_then(|entry| entry.values.get_index(col))
        .map(|(_, value)| value)
}

fn schema_for_header<'a>(header: &Header, schemas: &'a IndexMap<Header, Schema>) -> &'a Schema {
    lazy_static::lazy_static! {
        pub static ref GENERIC : Schema = [
            ("data".into(), tocs::tbl::AtomicType::X(0))
        ].into_iter().collect();
    };
    schemas.get(header).unwrap_or(&GENERIC)
}

fn export_csv(path: &std::path::Path, schema: &Schema, data: &[AttributeValues]) -> std::io::Result<()> {
    log::debug!("export data as CSV to '{}'", path.display());
    let mut csv = csv::WriterBuilder::new().quote_style(csv::QuoteStyle::Always).from_path(path)?;

    log::debug!("writing csv header");
    for attr in schema.keys() {
        csv.write_field(attr.as_bytes())?;
    }
    // end of record
    let eor: [&str; 0] = [];
    csv.write_record(eor)?;

    log::debug!("writing csv records");
    for entry in data {
        // one field for each attribute value
        for val in entry.values() {
            csv.write_field(val.to_string().as_bytes())?;
        }
        // end of record
        let eor: [&str; 0] = [];
        csv.write_record(eor)?;
    }
    Ok(())
}

fn import_csv(path: &std::path::Path, schema: &Schema) -> Result<std::vec::Vec<AttributeValues>, String> {
    let mut csv = csv::ReaderBuilder::new().has_headers(true).from_path(path).map_err(|e| e.to_string())?;

    // read headers and compare to the expected schema
    // note that the order of columns does not matter
    let expected_headers: std::collections::BTreeSet<_> = schema.keys().map(|s| s.as_ref()).collect();
    let csv_headers = csv.headers().map_err(|e| e.to_string())?;
    let actual_headers: std::collections::BTreeSet<_> = csv_headers.iter().collect();
    if let Some(h) = expected_headers.difference(&actual_headers).next() {
        return Err(format!("expected header '{}' is missing from CSV", h));
    }
    if let Some(h) = actual_headers.difference(&expected_headers).next() {
        return Err(format!("unexpected header '{}' present in CSV", h));
    }
    if actual_headers.len() != csv_headers.len() {
        return Err("CSV file has duplicate column header".to_string()); // TODO: should probably list which ones they are
    }
    let csv_column_index: std::collections::HashMap<_, usize> = csv_headers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            (
                // we need to drop csv_headers, so we reference the schema string instead of the csv header one
                schema.get_key_value(s).expect("csv headers should equal schema headers").0,
                i,
            )
        })
        .collect();
    drop(expected_headers);

    // read rows
    let mut result = vec![];
    for row in csv.records() {
        let row = row.map_err(|e| format!("error reading row: {}", e))?;
        if row.len() != schema.len() {
            return Err(format!("row has {} entries instead of the expected {}", row.len(), schema.len()));
        }
        let mut entry = AttributeValues::new();
        for (name, r#type) in schema.iter() {
            let s = row
                .get(*csv_column_index.get(name).expect("csv headers should equal schema attributes"))
                .expect("number of columns should match");
            let v = tocs::tbl::AtomicValue::from_str_nonstrict(r#type, s).map_err(|_| format!("invalid value '{}' for attribute '{}' (expected {})", s, name, r#type.describe()))?;
            entry.insert(name.clone(), v);
        }
        result.push(entry);
    }

    Ok(result)
}

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    let CliArgs { game_version, file } = CliArgs::parse();

    let appstate = match (game_version, file) {
        (Some(version), Some(path)) => {
            log::debug!("parsing {} as a {:?} file", path.display(), version);
            // try loading data from file
            match tocs::tbl::read_file(&path, version) {
                tocs::tbl::ReadTblResult::Ok(tbl) => AppData {
                    tbl_history: TblStateHistory::new(tbl),
                    active_entry_type: None,
                    tbl_version: version,
                    path,
                    warnings: vec![],
                },
                tocs::tbl::ReadTblResult::Warn { tbl, warnings } => {
                    AppData {
                        tbl_history: TblStateHistory::new(tbl),
                        active_entry_type: None,
                        tbl_version: version,
                        path,
                        warnings,
                    }
                    // TODO: if errors are harmless (e.g. all length bytes & all headers are in the schema), don't alert
                }
                tocs::tbl::ReadTblResult::Err(e) => {
                    log::error!("Unable to read file: {e}");
                    return;
                }
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            log::error!("game version and file must be given both (or neither)");
            return;
        }
        (None, None) => AppData {
            tbl_history: TblStateHistory::default(),
            active_entry_type: None,
            tbl_version: tocs::tbl::Version::CS1En,
            path: std::path::PathBuf::new(),
            warnings: vec![],
        },
    };

    log::debug!("starting app");
    let app = App::new(appstate);
    app.run();
}

fn try_get_selected_cell(table: &fltk::table::Table) -> Option<(i32, i32)> {
    let (row1, col1, row2, col2) = table.try_get_selection()?;
    if row1 == row2 && col1 == col2 {
        Some((row1, col1))
    } else {
        None
    }
}

fn try_get_selected_row(table: &fltk::table::Table) -> Option<i32> {
    let (row1, col1, row2, col2) = table.try_get_selection()?;
    if row1 == row2 && col1 == 0 && col2 == table.cols() - 1 {
        Some(row1)
    } else {
        None
    }
}

fn draw_header(txt: &str, x: i32, y: i32, w: i32, h: i32) {
    fltk::draw::push_clip(x, y, w, h);
    fltk::draw::draw_box(fltk::enums::FrameType::ThinUpBox, x, y, w, h, fltk::enums::Color::FrameDefault);
    fltk::draw::set_draw_color(fltk::enums::Color::Black);
    fltk::draw::draw_text2(txt, x, y, w, h, fltk::enums::Align::Center);
    fltk::draw::pop_clip();
}

pub enum CellType {
    EvenRow,
    OddRow,
    Selected, // currently selected
    Changed,  // cell was changed since last save; TODO: actually make use of this
}

fn draw_data(value: &tocs::tbl::AtomicValue, x: i32, y: i32, w: i32, h: i32, celltype: CellType) {
    fltk::draw::push_clip(x, y, w, h);
    fltk::draw::set_draw_color(fltk::enums::Color::from_u32(match celltype {
        CellType::OddRow => 0x00_ff_ff_ff,
        CellType::EvenRow => 0x00_F0_F0_F0,
        CellType::Selected => 0x00_D0_D0_D0,
        CellType::Changed => 0x00_C0_C0_FF,
    }));
    fltk::draw::draw_rectf(x, y, w, h);
    fltk::draw::set_draw_color(fltk::enums::Color::Gray0);
    //fltk::draw::set_font(enums::Font::Helvetica, 14);
    let text_alignment = match value.r#type() {
        tocs::tbl::AtomicType::I8
        | tocs::tbl::AtomicType::I16
        | tocs::tbl::AtomicType::I32
        | tocs::tbl::AtomicType::U8
        | tocs::tbl::AtomicType::U16
        | tocs::tbl::AtomicType::U32
        | tocs::tbl::AtomicType::F32 => fltk::enums::Align::Right,
        tocs::tbl::AtomicType::CUtf8 => fltk::enums::Align::Left,
        tocs::tbl::AtomicType::X(_) => fltk::enums::Align::Left,
    };

    fltk::draw::draw_rect(x, y, w, h);

    let (x, w) = if w > 5 { (x + 2, w - 4) } else { (x, w) };
    let (y, h) = if h > 5 { (y + 2, h - 4) } else { (y, h) };
    fltk::draw::draw_text2(&value.to_string(), x, y, w, h, text_alignment);
    fltk::draw::pop_clip();
}
