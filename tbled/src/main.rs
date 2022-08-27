use fltk::prelude::GroupExt as _;
use fltk::prelude::TableExt as _;
use fltk::prelude::WidgetExt as _;

fn partition_entries_with_headers(
    entries: std::vec::Vec<(String, indexmap::IndexMap<String, tocs::tbl::AtomicValue<String>>)>,
) -> indexmap::IndexMap<String, std::vec::Vec<indexmap::IndexMap<String, tocs::tbl::AtomicValue<String>>>> {
    let mut entries_for_header: indexmap::IndexMap<String, std::vec::Vec<indexmap::IndexMap<String, tocs::tbl::AtomicValue<String>>>> = indexmap::IndexMap::new();
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

type Attributes = indexmap::IndexMap<String, tocs::tbl::AtomicType>;
type AttributeValues = indexmap::IndexMap<String, tocs::tbl::AtomicValue<String>>;

#[derive(Clone, Debug)]
enum Msg {
    UpdateTable,
}

struct Widgets {
    header_selector: fltk::misc::InputChoice,
    table: fltk::table::Table,
}

#[derive(Clone)] // super disgusting, but i'm in despair right now
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

        let tbl_entries = load_tbl_file();
        let mut loaded_table_name = tbl_entries.iter().next().map(|(k, v)| k.to_string());

        let mut header_selector = fltk::misc::InputChoice::default();
        for hdr in tbl_entries.keys() {
            header_selector.add(&hdr);
        }
        if let Some((hdr, entries)) = tbl_entries.iter().next() {
            header_selector.set_value(hdr);
            loaded_table_name = Some(hdr.to_string());
        }
        header_selector.emit(sender, Msg::UpdateTable);
        flex.set_size(&mut header_selector, 26);

        let mut table = fltk::table::Table::default();
        table.set_col_header(true);
        table.set_col_width_all(80);
        table.set_col_resize(true);
        table.set_row_header(false);
        table.set_row_resize(true);

        table.draw_cell(|table, ctx, row, col, x, y, w, h| {
            // Called when the table is drawn then when it's redrawn due to events
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
                //fltk::table::TableContext::RowHeader => draw_header(&format!("{}", row + 1), x, y, w, h),        // Row titles
                fltk::table::TableContext::Cell => {
                    draw_data(&value, x, y, w, h, table.is_selected(row, col));
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
            widgets: Widgets { header_selector, table },
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
        let table = &mut self.widgets.table;

        let (nrows, ncols) = {
            let mut state = TBL.lock().unwrap();
            let header = state.active.as_ref();
            let ncols = header.and_then(|hdr| tocs::tbl::SCHEMAS_CS3_ATOMIC.get(hdr)).and_then(|attrs| attrs.len().try_into().ok()).unwrap_or(1);
            let nrows = header.and_then(|hdr| state.entries.get(hdr)).and_then(|rows| rows.len().try_into().ok()).unwrap_or(0);
            (nrows, ncols)
        };
        table.set_rows(nrows);
        table.set_cols(ncols);
    }
}

fn load_tbl_file() -> indexmap::IndexMap<String, std::vec::Vec<AttributeValues>> {
    let mut f = std::fs::File::open("../private-test-data/cs3/data/text/dat_en/t_status.tbl").unwrap();
    let mut log = tocs::tbl::WarningLogger::new(true);
    let (headers, all_entries) = tocs::tbl::deserialize_tbl_v2(&mut f, &mut log, true, &tocs::tbl::SCHEMAS_CS3_ATOMIC).unwrap();
    drop(f);
    let mut tbl_entries = partition_entries_with_headers(all_entries);
    for (header, _) in headers {
        tbl_entries.entry(header).or_insert_with(|| vec![]);
    }
    tbl_entries
}

fn main() {
    {
        let entries = load_tbl_file();
        let active = entries.iter().next().map(|(k, v)| k.to_string());
        *TBL.lock().unwrap() = AppState { entries, active };
    }
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
fn draw_data(value: &tocs::tbl::AtomicValue<String>, x: i32, y: i32, w: i32, h: i32, selected: bool) {
    fltk::draw::push_clip(x, y, w, h);
    if selected {
        fltk::draw::set_draw_color(fltk::enums::Color::from_u32(0x00D3_D3D3));
    } else {
        fltk::draw::set_draw_color(fltk::enums::Color::White);
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
