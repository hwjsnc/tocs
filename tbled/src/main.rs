use fltk::prelude::GroupExt as _;
use fltk::prelude::InputExt as _;
use fltk::prelude::TableExt as _;
use fltk::prelude::WidgetExt as _;

type Attributes = indexmap::IndexMap<String, tocs::tbl::AtomicType>;
type AttributeValues = indexmap::IndexMap<String, tocs::tbl::AtomicValue<String>>;

// Given rows in any order, collect all rows with the same header together
fn partition_entries_with_headers(headers: indexmap::IndexMap<String, u32>, entries: std::vec::Vec<(String, AttributeValues)>) -> indexmap::IndexMap<String, std::vec::Vec<AttributeValues>> {
    let mut entries_for_header: indexmap::IndexMap<String, std::vec::Vec<AttributeValues>> = headers.into_iter().map(|(hdr, _)| (hdr, vec![])).collect();
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

#[derive(Debug, Clone, Copy)]
pub enum TblVersion {
    CS1,
    CS2,
    CS3,
}

lazy_static::lazy_static! {
    pub static ref VERSION_SELECTOR_OPTIONS: indexmap::IndexMap<&'static str, TblVersion> = [
        ("CS1 (XSeed PC, EN)", TblVersion::CS1),
        ("CS2 (XSeed PC)", TblVersion::CS2),
        ("CS3 (NISA PC)", TblVersion::CS3),
    ].into_iter().collect();
}

#[derive(Clone, Debug)]
enum Msg {
    SelectAndLoadFile, // user pushed button to load a file
    SaveFile,          // user pushed button to save to the same file
    SelectAndSaveFile, // user pushed button to save a file
    FileReloaded,      // user loaded a different tbl file and we have to update all widgets accordingly
    UpdateTable,       // user selected a different header; update the table widget
    TableEvent,        // An arbitrary event on the table
    DoneEditing,       // user is done editing, now the value should be updated
}

struct Widgets {
    tbl_version_selector: fltk::misc::InputChoice,
    //filename_output: fltk::output::Output,
    input: fltk::input::Input,
    header_selector: fltk::misc::InputChoice,
    table: fltk::table::Table,
}

pub struct AppState {
    schemas: indexmap::IndexMap<String, Attributes>,
    entries: indexmap::IndexMap<String, std::vec::Vec<AttributeValues>>,
    active: Option<String>,
    //edited: std::collections::HashSet<(i32, i32)>, // edited since last save
}

lazy_static::lazy_static! {
    pub static ref STATE: std::sync::Mutex<AppState> = std::sync::Mutex::new(AppState {
        schemas: indexmap::IndexMap::new(),
        entries: indexmap::IndexMap::new(),
        active: None,
        //edited: std::collections::HashSet::new(),
    });
}

struct App {
    app: fltk::app::App,
    receiver: fltk::app::Receiver<Msg>,
    widgets: Widgets,
}

impl App {
    pub fn new() -> Self {
        let (w, h) = (1280, 720);
        let app = fltk::app::App::default();
        let (sender, receiver) = fltk::app::channel();
        let mut win = fltk::window::Window::default().with_size(w, h).with_label("tbled");
        let mut flex = fltk::group::Flex::default().size_of_parent();
        flex.set_type(fltk::group::FlexType::Column);

        let mut flex_row = fltk::group::Flex::default().size_of_parent();
        flex_row.set_type(fltk::group::FlexType::Row);

        let mut tbl_version_selector = fltk::misc::InputChoice::default();
        for (s, _) in VERSION_SELECTOR_OPTIONS.iter() {
            tbl_version_selector.add(s);
        }
        tbl_version_selector.set_value_index(0);
        tbl_version_selector.input().set_align(fltk::enums::Align::Left);
        tbl_version_selector.input().set_readonly(true);
        flex_row.set_size(&mut tbl_version_selector, 160);

        let mut load_button = fltk::button::Button::default().with_label("Select && load file");
        load_button.emit(sender.clone(), Msg::SelectAndLoadFile);
        flex_row.set_size(&mut load_button, 140);

        let mut save_button = fltk::button::Button::default().with_label("Save");
        save_button.emit(sender.clone(), Msg::SaveFile);
        flex_row.set_size(&mut save_button, 60);

        let mut save_button = fltk::button::Button::default().with_label("Select && save file");
        save_button.emit(sender.clone(), Msg::SelectAndSaveFile);
        flex_row.set_size(&mut save_button, 140);

        let mut widget = fltk::widget::Widget::default(); // just for spacing

        //let mut label = fltk::output::Output::default();//.with_label("tbl header:");
        //label.set_value("tbl header: ");
        let mut label = fltk::widget::Widget::default().with_label("tbl header:");
        flex_row.set_size(&mut label, 100);

        let mut header_selector = fltk::misc::InputChoice::default().with_label("tbl header");
        header_selector.emit(sender.clone(), Msg::UpdateTable);
        header_selector.input().set_readonly(true);
        flex_row.set_size(&mut header_selector, 200);

        flex_row.end();
        flex.set_size(&mut flex_row, 26);

        let mut input = fltk::input::Input::default();
        input.set_trigger(fltk::enums::CallbackTrigger::EnterKeyAlways);
        input.emit(sender.clone(), Msg::DoneEditing);
        flex.set_size(&mut input, 26);

        let mut table = fltk::table::Table::default();
        table.set_col_header(true);
        table.set_col_width_all(40);
        table.set_col_resize(true);
        table.set_row_header(true);
        table.set_row_resize(true);

        table.emit(sender.clone(), Msg::TableEvent);

        table.draw_cell(|table, ctx, row, col, x, y, w, h| {
            log::trace!("table draw");
            // called when the table is drawn (both the first time and due to events)
            let state = STATE.lock().unwrap();
            let header = if let Some(hdr) = state.active.as_ref() {
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
                //table::TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
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
                    draw_data(&value, x, y, w, h, celltype);
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
            widgets: Widgets {
                tbl_version_selector,
                header_selector,
                input,
                table,
            },
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
            Some(Msg::SelectAndLoadFile) => {
                // open a file browser dialog to select the file
                let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseFile);
                diag.show();
                let path = diag.filename();
                if path.as_os_str().is_empty() {
                    return;
                }
                // get tbl version from the ui element
                let version: &str = &self.widgets.tbl_version_selector.value().unwrap();
                let version: TblVersion = *VERSION_SELECTOR_OPTIONS.get(version).unwrap();
                {
                    // reset state related to tbl data
                    let mut state = STATE.lock().unwrap();
                    let AppState { active, schemas, entries } = &mut *state;
                    *active = None;
                    *schemas = schemas_for_version(version);
                    entries.clear();
                }
                {
                    // try loading data from file
                    let mut state = STATE.lock().unwrap();
                    let result = load_tbl_file(&path, version);
                    // TODO: handle warnings
                    match result {
                        Ok(entries) => {
                            state.entries = entries;
                        }
                        Err(e) => {
                            // TODO: better reporting
                            fltk::dialog::alert(400, 200, &format!("error loading file: {:?}", e));
                        }
                    }
                }
                // we need to update other ui elements
                self.update(&Some(Msg::FileReloaded));
            }
            Some(Msg::SaveFile) => {
                fltk::dialog::alert(400, 200, "saving in place is not yet implemented");
            }
            Some(Msg::SelectAndSaveFile) => {
                // open a file browser dialog to select the file
                let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseSaveFile);
                diag.show();
                let path = diag.filename();
                if path.as_os_str().is_empty() {
                    return;
                }
                // get tbl version from the ui element
                let version: &str = &self.widgets.tbl_version_selector.value().unwrap();
                let version: TblVersion = *VERSION_SELECTOR_OPTIONS.get(version).unwrap();
                {
                    // try saving data to file
                    let mut state = STATE.lock().unwrap();
                    let result = save_tbl_file(&path, version, &state.entries);
                    match result {
                        Ok(entries) => {
                            // TODO: clear changed cell flags
                        }
                        Err(e) => {
                            // TODO: better reporting
                            fltk::dialog::alert(400, 200, &format!("error loading file: {:?}", e));
                        }
                    }
                }
            }
            Some(Msg::FileReloaded) => {
                // reset header selector
                {
                    let mut state = STATE.lock().unwrap();
                    state.active = self.widgets.header_selector.value().and_then(|hdr| state.entries.get(&hdr).map(|_| hdr.to_string()));
                    self.widgets.header_selector.clear();
                    for hdr in state.entries.keys() {
                        self.widgets.header_selector.add(&hdr);
                    }
                    if let Some((hdr, entries)) = state.entries.iter().next() {
                        self.widgets.header_selector.set_value(hdr);
                        state.active = Some(hdr.to_string());
                    }
                }
                // Note: mutex must be free for this function because it triggers table redraws:
                self.update(&Some(Msg::UpdateTable));
            }
            Some(Msg::UpdateTable) => {
                let mut state = STATE.lock().unwrap();
                state.active = self.widgets.header_selector.value().and_then(|hdr| state.entries.get(&hdr).map(|_| hdr.to_string()));
                if let Some(hdr) = &state.active {
                    // width of each column = max(header width, width of each entry in that column)
                    // width of each row = max(width of all entries in all rows)
                    let mut column_widths: std::vec::Vec<i32> = if let Some(attrs) = state.schemas.get(hdr) {
                        // known schema, get text widths of the attributes (names)
                        attrs.iter().map(|(hdr, _)| fltk::draw::measure(hdr, false).0).collect()
                    } else {
                        // no known schema
                        vec![10]
                    };
                    let entries = state.entries.get(hdr).unwrap();
                    let nrows = entries.len().try_into().unwrap();
                    let mut row_height = 1;
                    for row in entries {
                        for (i, (_, val)) in row.iter().enumerate() {
                            let (w, h) = fltk::draw::measure(&tocs::tbl::atomic_value_to_string(&val), false);
                            column_widths[i] = std::cmp::max(column_widths[i], w);
                            row_height = std::cmp::max(row_height, h);
                        }
                    }
                    drop(state);
                    // Note: mutex must be free for this function because it triggers table redraws:
                    self.update_table(&column_widths, nrows, row_height);
                }
            }
            Some(Msg::TableEvent) => {
                match fltk::app::event() {
                    fltk::enums::Event::Released if fltk::app::event_clicks() => {
                        // find currently selected cell (if it's exactly one)
                        let table = &mut self.widgets.table;
                        log::debug!("double click on table");
                        match table.try_get_selection() {
                            Some((row, col, row2, col2)) if row == row2 && col == col2 => {
                                // copy table value to input
                                log::debug!("double click on cell");
                                let mut state = STATE.lock().unwrap();
                                self.widgets.input.set_value(&tocs::tbl::atomic_value_to_string(get_cell_value(&mut *state, row, col).unwrap()));
                                self.widgets.input.take_focus().unwrap();
                                self.widgets.input.redraw();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Some(Msg::DoneEditing) => {
                // find the currently selected cell (if it's exactly one)
                let input = &mut self.widgets.input;
                let table = &mut self.widgets.table;
                match table.try_get_selection() {
                    Some((row, col, row2, col2)) if row == row2 && col == col2 => {
                        // copy text input to selected cell
                        {
                            let mut state = STATE.lock().unwrap();
                            let text = input.value();
                            let cell_value = get_cell_value(&mut *state, row, col).unwrap();
                            let ty = &tocs::tbl::atomic_type(cell_value);
                            if let Ok(value) = tocs::tbl::try_atomic_value_from_str(&ty, &text) {
                                *cell_value = value;
                            } else {
                                fltk::dialog::alert(400, 200, &format!("invalid input for cell, expected a {}", tocs::tbl::atomic_type_describe(&ty)));
                            }
                        }
                        table.redraw();
                    }
                    _ => {}
                }
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

fn get_cell_value<'a>(state: &'a mut AppState, row: i32, col: i32) -> Option<&'a mut tocs::tbl::AtomicValue<String>> {
    let row: usize = row.try_into().unwrap();
    let col: usize = col.try_into().unwrap();
    let AppState { active, schemas, entries } = state;
    active
        .as_ref()
        .and_then(|hdr: &String| entries.get_mut(*&hdr))
        .and_then(|rows| rows.get_mut(row))
        .and_then(|row| row.get_index_mut(col))
        .map(|(_, val)| val)
}

fn schemas_for_version(version: TblVersion) -> indexmap::IndexMap<String, Attributes> {
    match version {
        TblVersion::CS1 => tocs::tbl::SCHEMAS_CS1_ATOMIC.clone(),
        TblVersion::CS2 => tocs::tbl::SCHEMAS_CS2_ATOMIC.clone(),
        TblVersion::CS3 => tocs::tbl::SCHEMAS_CS3_ATOMIC.clone(),
    }
}

fn load_tbl_file(path: &std::path::Path, version: TblVersion) -> Result<indexmap::IndexMap<String, std::vec::Vec<AttributeValues>>, tocs::tbl::DeserializeError> {
    // TODO: handle warnings
    let mut log = tocs::tbl::WarningLogger::new(true);
    match version {
        TblVersion::CS1 => tocs::tbl::load_file_v1(path, &mut log, true, &tocs::tbl::SCHEMAS_CS1_ATOMIC).map(|entries| (indexmap::IndexMap::new(), entries)),
        TblVersion::CS2 => tocs::tbl::load_file_v2(path, &mut log, true, &tocs::tbl::SCHEMAS_CS2_ATOMIC),
        TblVersion::CS3 => tocs::tbl::load_file_v2(path, &mut log, true, &tocs::tbl::SCHEMAS_CS3_ATOMIC),
    }
    .map(|(headers, entries)| partition_entries_with_headers(headers, entries))
}

fn save_tbl_file(path: &std::path::Path, version: TblVersion, entries: &indexmap::IndexMap<String, std::vec::Vec<AttributeValues>>) -> Result<usize, tocs::tbl::SerializeError> {
    // TODO: handle warnings
    let headers: std::vec::Vec<&str> = entries.keys().map(|hdr| hdr.as_ref()).collect();
    let mut entries: std::vec::Vec<(&str, AttributeValues)> = entries.iter().map(|(hdr, entries)| entries.iter().map(|e| (hdr.as_ref(), e.clone()))).flatten().collect();
    match version {
        TblVersion::CS1 => tocs::tbl::save_file_v1(path, &entries),
        TblVersion::CS2 => tocs::tbl::save_file_v2(path, &headers, &entries),
        TblVersion::CS3 => tocs::tbl::save_file_v2(path, &headers, &entries),
    }
}

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    let app = App::new();
    app.run();
    return;
}

fn draw_header(txt: &str, x: i32, y: i32, w: i32, h: i32) {
    fltk::draw::push_clip(x, y, w, h);
    fltk::draw::draw_box(fltk::enums::FrameType::ThinUpBox, x, y, w, h, fltk::enums::Color::FrameDefault);
    fltk::draw::set_draw_color(fltk::enums::Color::Black);
    //fltk::draw::set_font(enums::Font::Helvetica, 14);
    fltk::draw::draw_text2(txt, x, y, w, h, fltk::enums::Align::Center);
    fltk::draw::pop_clip();
}

pub enum CellType {
    EvenRow,
    OddRow,
    Selected, // currently selected
    Changed,  // cell was changed since last save
}

// The selected flag sets the color of the cell to a grayish color, otherwise white
fn draw_data(value: &tocs::tbl::AtomicValue<String>, x: i32, y: i32, w: i32, h: i32, celltype: CellType) {
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
    fltk::draw::draw_text2(&tocs::tbl::atomic_value_to_string(value), x, y, w, h, text_alignment);
    fltk::draw::pop_clip();
}
