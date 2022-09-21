#![windows_subsystem = "windows"] // don't open terminal on windows

mod warn_diag;

use fltk::prelude::GroupExt as _;
use fltk::prelude::InputExt as _;
use fltk::prelude::MenuExt as _;
use fltk::prelude::TableExt as _;
use fltk::prelude::WidgetExt as _;

use clap::Parser;

use indexmap::IndexMap;
use smol_str::SmolStr;

type Attributes = IndexMap<SmolStr, tocs::tbl::AtomicType>;
type AttributeValues = IndexMap<SmolStr, tocs::tbl::AtomicValue>;

// Given rows in any order, collect all rows with the same header together
fn partition_entries_with_headers(headers: std::vec::Vec<SmolStr>, entries: std::vec::Vec<(SmolStr, AttributeValues)>) -> IndexMap<SmolStr, std::vec::Vec<AttributeValues>> {
    let mut entries_for_header: IndexMap<SmolStr, std::vec::Vec<AttributeValues>> = headers.into_iter().map(|hdr| (hdr, vec![])).collect();
    for (hdr, data) in entries.into_iter() {
        match entries_for_header.entry(hdr) {
            indexmap::map::Entry::Occupied(mut e) => {
                e.get_mut().push(data);
            }
            indexmap::map::Entry::Vacant(e) => {
                e.insert(vec![data]);
            }
        }
    }
    entries_for_header
}

#[derive(Debug, Parser)]
struct CliArgs {
    #[clap(short, long)]
    game_version: Option<tocs::tbl::Version<'static>>,
    #[clap(short, long)]
    file: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug)]
enum Msg {
    SelectAndLoadFile(tocs::tbl::Version<'static>), // user wants to load a file
    SaveFile,                                       // user wants to save to the same file
    SelectAndSaveFile,                              // user wants to save a file
    ExportCSV,                                      // user wants to export file to csv
    ShowWarnings,                                   // user wants to see warnings
    FileReloaded,                                   // user loaded a different tbl file and we have to update all widgets accordingly
    UpdateTable,                                    // user selected a different header; update the table widget
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
    entries: IndexMap<SmolStr, std::vec::Vec<AttributeValues>>,
    tbl_version: tocs::tbl::Version<'static>,
    path: std::path::PathBuf,
    active_entry_type: Option<SmolStr>,
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
            let state = state_clone.lock().unwrap();
            let header = if let Some(hdr) = state.active_entry_type.as_ref() {
                hdr
            } else {
                return;
            };
            let rows = if let Some(rows) = state.entries.get(header) {
                rows
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
            let row_values: &AttributeValues = if let Some(row) = rows.get(row_idx) {
                row
            } else {
                return;
            };
            let (attribute, value) = if let Some(tup) = row_values.get_index(col_idx) {
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
                    }
                    /* else if changed: CellType::Changed */
                    else if row % 2 == 0 {
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
                let path = diag.filename();
                if path.as_os_str().is_empty() {
                    return;
                }
                {
                    // reset state related to tbl data
                    let mut state = self.state.lock().unwrap();
                    let AppData {
                        active_entry_type,
                        tbl_version,
                        path: _,
                        entries,
                        warnings,
                    } = &mut *state;
                    *active_entry_type = None;
                    *tbl_version = *version;
                    entries.clear();
                    warnings.clear();
                    // try loading data from file
                    match tocs::tbl::read_file(&path, *version) {
                        tocs::tbl::ReadTblResult::Ok(tbl) => {
                            let headers = tbl.headers().as_ref().iter().map(SmolStr::from).collect();
                            state.entries = partition_entries_with_headers(headers, tbl.into_entries());
                            state.path = path;
                        }
                        tocs::tbl::ReadTblResult::Warn { tbl, warnings } => {
                            let headers = tbl.headers().as_ref().iter().map(SmolStr::from).collect();
                            state.warnings = warnings;
                            state.entries = partition_entries_with_headers(headers, tbl.into_entries());
                            state.path = path;
                            // TODO: if errors are harmless (e.g. all length bytes & all headers are in the schema), don't alert
                            fltk::dialog::alert_default(
                                "Warning: Some errors/inconsistencies in the file were detected. Please look at them under the Warnings menu option and ensure that they are harmless",
                            );
                        }
                        tocs::tbl::ReadTblResult::Err(e) => {
                            fltk::dialog::alert_default(&format!("error loading file: {}", e));
                            state.path = std::path::PathBuf::new();
                        }
                    }
                }
                // we need to update other ui elements
                self.update(&Some(Msg::FileReloaded));
            }
            Some(Msg::SaveFile) => {
                // try saving data to file
                let state = self.state.lock().unwrap();
                let result = save_tbl_file(&state.path, state.tbl_version, &state.entries);
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
                let result = save_tbl_file(&path, state.tbl_version, &state.entries);
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
                let state = self.state.lock().unwrap();
                if let Some(hdr) = state.active_entry_type.as_ref() {
                    // open a file browser dialog to select the file
                    let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseSaveFile);
                    diag.set_title("Select .tbl file to save to ...");
                    diag.set_preset_file(&format!("{}.csv", hdr));
                    diag.show();
                    let path = diag.filename();
                    if path.as_os_str().is_empty() {
                        return;
                    }
                    match export_csv(&path, state.entries.get(hdr).unwrap()) {
                        Ok(_) => {}
                        Err(e) => {
                            fltk::dialog::alert_default(&format!("error exporting to csv: {:?}", e));
                        }
                    }
                } else {
                    fltk::dialog::alert_default("no entry type selected, can't export");
                }
            }
            Some(Msg::FileReloaded) => {
                {
                    let mut state = self.state.lock().unwrap();
                    let AppData {
                        active_entry_type,
                        tbl_version: _,
                        path: _,
                        entries,
                        warnings,
                    } = &mut *state;
                    // if there are warnings, show button to display them
                    if warnings.is_empty() {
                        self.widgets.menu.find_item("&Warnings/Show warnings...\t").unwrap().deactivate();
                    } else {
                        self.widgets.menu.find_item("&Warnings/Show warnings...\t").unwrap().activate();
                    }
                    // reset header selector
                    *active_entry_type = self.widgets.header_selector.value().and_then(|hdr| entries.get(&hdr as &str).map(|_| SmolStr::new(hdr)));
                    self.widgets.header_selector.clear();
                    for hdr in state.entries.keys() {
                        self.widgets.header_selector.add(hdr);
                    }
                    if let Some((hdr, _)) = state.entries.iter().next() {
                        self.widgets.header_selector.set_value(hdr);
                        state.active_entry_type = Some(hdr.clone());
                    }
                }
                // Note: mutex must be free for this function because it triggers table redraws:
                self.update(&Some(Msg::UpdateTable));
            }
            Some(Msg::UpdateTable) => {
                let mut state = self.state.lock().unwrap();
                log::trace!("boop");
                state.active_entry_type = self.widgets.header_selector.value().and_then(|hdr| state.entries.get(&hdr as &str).map(|_| SmolStr::new(hdr)));
                if let Some(hdr) = &state.active_entry_type {
                    log::trace!("beep");
                    // width of each column = max(header width, width of each entry in that column)
                    // width of each row = max(width of all entries in all rows)
                    let attrs = schema_for_header(hdr, state.tbl_version.schemas());
                    let mut column_widths: std::vec::Vec<i32> = attrs.iter().map(|(hdr, _)| fltk::draw::measure(hdr, false).0).collect();
                    let entries = state.entries.get(hdr).unwrap();
                    let nrows = entries.len().try_into().unwrap();
                    let mut row_height = 1;
                    for row in entries {
                        for (i, (_, val)) in row.iter().enumerate() {
                            let (w, h) = fltk::draw::measure(tocs::tbl::atomic_value_to_string(val).as_ref(), false);
                            column_widths[i] = std::cmp::max(column_widths[i], w);
                            row_height = std::cmp::max(row_height, h);
                        }
                    }
                    log::trace!("{} {} {}", column_widths.len(), nrows, row_height);
                    drop(state);
                    // Note: mutex must be free for this function because it triggers table redraws:
                    self.update_table(&column_widths, nrows, row_height);
                }
            }
            Some(Msg::DuplicateEntry) => {
                if let Some(row) = try_get_selected_row(&self.widgets.table) {
                    let row: usize = row.try_into().unwrap();
                    let mut state = self.state.lock().unwrap();
                    let AppData { active_entry_type, entries, .. } = &mut *state;
                    if let Some(hdr) = active_entry_type {
                        let active_entries = entries.get_mut(hdr).unwrap();
                        active_entries.insert(row, active_entries[row].clone());
                    }
                    drop(state);
                    // Note: mutex must be free for this function because it triggers table redraws:
                    self.update(&Some(Msg::UpdateTable));
                } else {
                    fltk::dialog::alert_default("please select exactly one row");
                }
            }
            Some(Msg::DeleteEntry) => {
                if let Some(row) = try_get_selected_row(&self.widgets.table) {
                    let row: usize = row.try_into().unwrap();
                    let mut state = self.state.lock().unwrap();
                    let AppData { active_entry_type, entries, .. } = &mut *state;
                    if let Some(hdr) = active_entry_type {
                        let active_entries = entries.get_mut(hdr).unwrap();
                        active_entries.remove(row);
                    }
                    drop(state);
                    // Note: mutex must be free for this function because it triggers table redraws:
                    self.update(&Some(Msg::UpdateTable));
                } else {
                    fltk::dialog::alert_default("please select exactly one row");
                }
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
                            self.widgets.input.set_value(tocs::tbl::atomic_value_to_string(get_cell_value(&mut *state, row, col).unwrap()).as_ref());
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
                        let mut state = self.state.lock().unwrap();
                        let text = input.value();
                        let cell_value = get_cell_value(&mut *state, row, col).unwrap();
                        let ty = tocs::tbl::atomic_type(cell_value);
                        if let Ok(value) = tocs::tbl::try_atomic_value_from_str(&ty, &text) {
                            *cell_value = value;
                        } else {
                            fltk::dialog::alert_default(&format!("invalid input for cell, expected a {}", tocs::tbl::atomic_type_describe(&ty)));
                        }
                    }
                    table.redraw();
                }
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
}

fn get_cell_value(state: &mut AppData, row: i32, col: i32) -> Option<&mut tocs::tbl::AtomicValue> {
    let row: usize = row.try_into().unwrap();
    let col: usize = col.try_into().unwrap();
    let AppData { active_entry_type, entries, .. } = state;
    active_entry_type
        .as_ref()
        .and_then(|hdr| entries.get_mut(hdr))
        .and_then(|rows| rows.get_mut(row))
        .and_then(|row| row.get_index_mut(col))
        .map(|(_, val)| val)
}

fn schema_for_header<'a>(header: &str, schemas: &'a IndexMap<SmolStr, Attributes>) -> &'a Attributes {
    lazy_static::lazy_static! {
        pub static ref GENERIC : Attributes = [
            (SmolStr::from("data"), tocs::tbl::AtomicType::X(0))
        ].into_iter().collect();
    };
    schemas.get(header).unwrap_or(&GENERIC)
}

fn export_csv(path: &std::path::Path, data: &[AttributeValues]) -> std::io::Result<()> {
    log::debug!("export data as CSV to '{}'", path.display());
    let mut csv = csv::WriterBuilder::new().quote_style(csv::QuoteStyle::Always).from_path(path)?;
    log::debug!("writing csv header");
    if let Some(entry) = data.get(0) {
        for attr in entry.keys() {
            csv.write_field(attr.as_bytes())?;
        }
        // end of record
        let eor: [&str; 0] = [];
        csv.write_record(&eor)?;
    }
    log::debug!("writing csv records");
    for entry in data {
        // one field for each attribute value
        for val in entry.values() {
            csv.write_field(tocs::tbl::atomic_value_to_string(val).as_ref().as_bytes())?;
        }
        // end of record
        let eor: [&str; 0] = [];
        csv.write_record(&eor)?;
    }
    Ok(())
}

fn save_tbl_file(path: &std::path::Path, version: tocs::tbl::Version, entries: &IndexMap<SmolStr, std::vec::Vec<AttributeValues>>) -> Result<usize, tocs::tbl::SerializeError> {
    let headers: std::vec::Vec<SmolStr> = entries.keys().cloned().collect();
    let entries: std::vec::Vec<(SmolStr, AttributeValues)> = entries.iter().flat_map(|(hdr, entries)| entries.iter().map(|e| (hdr.clone(), e.clone()))).collect();
    let tbl = tocs::tbl::Tbl::new(version, headers, entries);
    tocs::tbl::write_file(path, &tbl)
}

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    let CliArgs { game_version, file } = CliArgs::parse();

    let appstate = match (game_version, file) {
        (Some(version), Some(path)) => {
            log::debug!("parsing {} as a {:?} file", path.display(), version);
            // try loading data from file
            match tocs::tbl::read_file(&path, version) {
                tocs::tbl::ReadTblResult::Ok(tbl) => {
                    let headers = tbl.headers().as_ref().iter().map(SmolStr::from).collect();
                    AppData {
                        active_entry_type: None,
                        tbl_version: version,
                        path,
                        entries: partition_entries_with_headers(headers, tbl.into_entries()),
                        warnings: vec![],
                    }
                }
                tocs::tbl::ReadTblResult::Warn { tbl, warnings } => {
                    let headers = tbl.headers().as_ref().iter().map(SmolStr::from).collect();
                    AppData {
                        active_entry_type: None,
                        tbl_version: version,
                        path,
                        entries: partition_entries_with_headers(headers, tbl.into_entries()),
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
            active_entry_type: None,
            tbl_version: tocs::tbl::Version::CS1En,
            path: std::path::PathBuf::new(),
            entries: IndexMap::new(),
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
    Changed,  // cell was changed since last save
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
    let text_alignment = match tocs::tbl::atomic_type(value) {
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
    fltk::draw::draw_text2(tocs::tbl::atomic_value_to_string(value).as_ref(), x, y, w, h, text_alignment);
    fltk::draw::pop_clip();
}
