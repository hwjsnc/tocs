// miscellaneous utility functions

use tocs::tbl::AtomicAttributeValues as AttributeValues;
use tocs::tbl::AtomicSchema as Schema;

pub fn load_file_dialog(filetype: &str) -> Option<std::path::PathBuf> {
    // open a file browser dialog to select the file
    let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseFile);
    diag.set_title(&format!("Select .{} file to load ...", filetype));
    diag.show();
    let path = diag.filename();
    if path.as_os_str().is_empty() {
        None
    } else {
        Some(path)
    }
}

pub fn save_file_dialog(filetype: &str, preset: &str) -> Option<std::path::PathBuf> {
    // open a file browser dialog to select the file
    let mut diag = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseSaveFile);
    diag.set_title(&format!("Select .{} file to save to ...", filetype));
    diag.set_preset_file(preset);
    diag.show();
    let path = diag.filename();
    if path.as_os_str().is_empty() {
        None
    } else {
        Some(path)
    }
}

pub fn export_csv(path: &std::path::Path, schema: &Schema, data: &[AttributeValues]) -> std::io::Result<()> {
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

pub fn import_csv(path: &std::path::Path, schema: &Schema) -> Result<std::vec::Vec<AttributeValues>, String> {
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
