use fltk::prelude::GroupExt as _;
use fltk::prelude::InputExt as _;
use fltk::prelude::TableExt as _;
use fltk::prelude::WidgetExt as _;

type Attributes = indexmap::IndexMap<String, tocs::tbl::AtomicType>;
type AttributeValues = indexmap::IndexMap<String, tocs::tbl::AtomicValue<String>>;

// Given rows in any order, collect all rows with the same header together
fn partition_entries_with_headers(entries: std::vec::Vec<(String, AttributeValues)>) -> indexmap::IndexMap<String, std::vec::Vec<AttributeValues>> {
    let mut entries_for_header: indexmap::IndexMap<String, std::vec::Vec<AttributeValues>> = indexmap::IndexMap::new();
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
}

struct Widgets {
    tbl_version_selector: fltk::misc::InputChoice,
    //filename_output: fltk::output::Output,
    header_selector: fltk::misc::InputChoice,
    table: fltk::table::Table,
}

pub struct AppState {
    entries: indexmap::IndexMap<String, std::vec::Vec<AttributeValues>>,
    active: Option<String>,
}

lazy_static::lazy_static! {
    pub static ref TBL: std::sync::Mutex<AppState> = std::sync::Mutex::new(AppState {
        entries: indexmap::IndexMap::new(),
        active: None,
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

        /*
        let tbl_entries = load_tbl_file();
        let mut loaded_table_name = tbl_entries.iter().next().map(|(k, v)| k.to_string());
        */

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

        /*
        let mut filename_output = fltk::output::Output::default();
        filename_output.set_value(""); // TODO
        flex_row.set_size(&mut filename_output, 400);
        */

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

        let mut table = fltk::table::Table::default();
        table.set_col_header(true);
        table.set_col_width_all(40);
        table.set_col_resize(true);
        table.set_row_header(true);
        table.set_row_resize(true);

        table.draw_cell(|table, ctx, row, col, x, y, w, h| {
            // called when the table is drawn (both the first time and due to events)
            let state = TBL.lock().unwrap();
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
                    draw_data(&value, x, y, w, h, table.is_selected(row, col), (row % 2) == 0);
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
                //filename_output,
                header_selector,
                table,
            },
        };

        this.update_table();

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
        // TODO
        match msg {
            Some(Msg::SelectAndLoadFile) => {
                let version: &str = &self.widgets.tbl_version_selector.value().unwrap();
                let version: TblVersion = *VERSION_SELECTOR_OPTIONS.get(version).unwrap();
                let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseFile);
                diag.show();
                let path = diag.filename();
                {
                    let mut state = TBL.lock().unwrap();
                    state.active = None;
                    state.entries = indexmap::IndexMap::new();
                }
                if !path.as_os_str().is_empty() {
                    let mut state = TBL.lock().unwrap();
                    state.active = None;
                    match load_tbl_file(&path, version) {
                        Ok(entries) => {
                            // TODO: handle warnings
                            state.entries = entries;
                        }
                        Err(e) => {
                            fltk::dialog::alert(400, 200, &format!("error loading file: {:?}", e));
                            // TODO: display
                        }
                    }
                }
                self.update(&Some(Msg::FileReloaded));
            }
            Some(Msg::SaveFile) => {
                println!("not yet implemented"); // TODO
            }
            Some(Msg::SelectAndSaveFile) => {
                println!("not yet implemented"); // TODO
            }
            Some(Msg::FileReloaded) => {
                {
                    // reset header selector
                    let mut state = TBL.lock().unwrap();
                    state.active = self.widgets.header_selector.value().and_then(|hdr| state.entries.get(&hdr).map(|_| hdr.to_string()));
                    for hdr in state.entries.keys() {
                        self.widgets.header_selector.add(&hdr);
                    }
                    if let Some((hdr, entries)) = state.entries.iter().next() {
                        self.widgets.header_selector.set_value(hdr);
                        state.active = Some(hdr.to_string());
                    }
                }
                self.update_table();
            }
            Some(Msg::UpdateTable) => {
                {
                    let mut state = TBL.lock().unwrap();
                    state.active = self.widgets.header_selector.value().and_then(|hdr| state.entries.get(&hdr).map(|_| hdr.to_string()));
                }
                self.update_table();
            }
            None => {
                // nothing
            }
        }
    }

    fn update_table(&mut self) {
        // don't call this function while holding an appstate mutex lock!
        let table = &mut self.widgets.table;

        let (nrows, ncols, column_widths) = {
            let mut state = TBL.lock().unwrap();
            let header = state.active.as_ref();
            let schema = header.and_then(|hdr| tocs::tbl::SCHEMAS_CS3_ATOMIC.get(hdr));
            let ncols = schema.and_then(|attrs| attrs.len().try_into().ok()).unwrap_or(1);
            let nrows = header.and_then(|hdr| state.entries.get(hdr)).and_then(|rows| rows.len().try_into().ok()).unwrap_or(0);
            let column_widths: Option<std::vec::Vec<_>> = schema.map(|attrs| {
                attrs
                    .iter()
                    .map(|(attr, ty)| match ty {
                        tocs::tbl::AtomicType::I8
                        | tocs::tbl::AtomicType::I16
                        | tocs::tbl::AtomicType::I32
                        | tocs::tbl::AtomicType::U8
                        | tocs::tbl::AtomicType::U16
                        | tocs::tbl::AtomicType::U32
                        | tocs::tbl::AtomicType::F32 => 60,
                        tocs::tbl::AtomicType::CUtf8 => 120,
                        tocs::tbl::AtomicType::X(_) => {
                            if attr == "data" {
                                1280
                            } else {
                                60
                            }
                        }
                    })
                    .collect()
            });
            (nrows, ncols, column_widths)
        };
        // note: we need to drop the mutex lock before changing the table.
        table.set_rows(nrows);
        table.set_cols(ncols);
        if let Some(cws) = column_widths {
            for (w, i) in cws.iter().zip(0..ncols) {
                table.set_col_width(i, *w);
            }
        } else if ncols > 0 {
            table.set_col_width_all(1260 / ncols);
        }
    }
}

fn load_tbl_file(path: &std::path::Path, version: TblVersion) -> Result<indexmap::IndexMap<String, std::vec::Vec<AttributeValues>>, tocs::tbl::DeserializeError> {
    // TODO: error handling
    // TODO: headers?
    let mut log = tocs::tbl::WarningLogger::new(true);
    match version {
        TblVersion::CS1 => tocs::tbl::load_file_v1(path, &mut log, true, &tocs::tbl::SCHEMAS_CS1_ATOMIC),
        TblVersion::CS2 => tocs::tbl::load_file_v2(path, &mut log, true, &tocs::tbl::SCHEMAS_CS2_ATOMIC).map(|(hdr, data)| data),
        TblVersion::CS3 => tocs::tbl::load_file_v2(path, &mut log, true, &tocs::tbl::SCHEMAS_CS3_ATOMIC).map(|(hdr, data)| data),
    }
    .map(|entries| partition_entries_with_headers(entries))
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

// The selected flag sets the color of the cell to a grayish color, otherwise white
fn draw_data(value: &tocs::tbl::AtomicValue<String>, x: i32, y: i32, w: i32, h: i32, selected: bool, even: bool) {
    fltk::draw::push_clip(x, y, w, h);
    if selected {
        fltk::draw::set_draw_color(fltk::enums::Color::from_u32(0x00_D0_D0_D0));
    } else {
        if even {
            fltk::draw::set_draw_color(fltk::enums::Color::White);
        } else {
            fltk::draw::set_draw_color(fltk::enums::Color::from_u32(0x00_F0_F0_F0));
        }
    }
    fltk::draw::draw_rectf(x, y, w, h);
    fltk::draw::set_draw_color(fltk::enums::Color::Gray0);
    //fltk::draw::set_font(enums::Font::Helvetica, 14);
    match value {
        tocs::tbl::AtomicValue::I8(val) => {
            fltk::draw::draw_text2(&format!("{val}"), x, y, w, h, fltk::enums::Align::Right);
        }
        tocs::tbl::AtomicValue::I16(val) => {
            fltk::draw::draw_text2(&format!("{val}"), x, y, w, h, fltk::enums::Align::Right);
        }
        tocs::tbl::AtomicValue::I32(val) => {
            fltk::draw::draw_text2(&format!("{val}"), x, y, w, h, fltk::enums::Align::Right);
        }
        tocs::tbl::AtomicValue::U8(val) => {
            fltk::draw::draw_text2(&format!("{val}"), x, y, w, h, fltk::enums::Align::Right);
        }
        tocs::tbl::AtomicValue::U16(val) => {
            fltk::draw::draw_text2(&format!("{val}"), x, y, w, h, fltk::enums::Align::Right);
        }
        tocs::tbl::AtomicValue::U32(val) => {
            fltk::draw::draw_text2(&format!("{val}"), x, y, w, h, fltk::enums::Align::Right);
        }
        tocs::tbl::AtomicValue::F32(val) => {
            fltk::draw::draw_text2(&format!("{val}"), x, y, w, h, fltk::enums::Align::Right);
        }
        tocs::tbl::AtomicValue::CUtf8(s) => {
            fltk::draw::draw_text2(s, x, y, w, h, fltk::enums::Align::Left);
        }
        tocs::tbl::AtomicValue::X(val) => {
            fltk::draw::draw_text2(&tocs::tbl::atomic_value_to_string(value), x, y, w, h, fltk::enums::Align::Left);
        }
    }
    fltk::draw::draw_rect(x, y, w, h);
    fltk::draw::pop_clip();
}
