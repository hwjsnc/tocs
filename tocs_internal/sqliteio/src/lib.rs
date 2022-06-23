extern crate rusqlite;

pub trait SqliteIO {
    /*
    Ideally we'd want to use arrays here, like:

    const N: usize
    fn columns() -> [&'static str; Self::N];
    fn sqlite_to_tup(&self) -> rusqlite::Result<[&dyn rusqlite::ToSql; N]>

    This is currently impossible: https://github.com/rust-lang/rust/issues/60551
    */

    /// Determines which table the entry belongs to:
    /// Generic entries with no known structure go in the "generic" table, other to tables that are named after the respective headers, e.g. table item for "item" entries.
    fn sqlite_table() -> &'static str;

    /// Column names, one for each entry attribute/struct
    /// E.g. for item, this would be vec!["id", "character_restriction", "flags", ...]
    ///
    /// Special care must be taken for repeated attributes, like effects. These must have differently named columns for each member of the sequence.
    /// The naming convention used here is "attribute name[number] x" for each attribute x of the repeated element.
    /// E.g. the single struct field `effects: [Effect; 5]` is turned into several columns, named "effects[0] id", "effects[0] data[0]", "effects[0] data[1]", "effects[0] data[2]", "effects[1] id", "effects[1] data[1]", etc.
    ///
    /// The following column names are illegal: file, rowid
    fn sqlite_columns() -> std::vec::Vec<String>;

    /// Converts entry data into a sqlite tuple following the schema from the `columns` function
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql>;

    /// Attempts to convert values from an sql row starting at a given column index.
    /// On success, returns the value corresponding to the values in the row, and the index of the column after the last read column.
    /// (This index may or may not equal one past the final column).
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)>
    where
        Self: Sized;
}

// Blanket implementations for primitive types
// Should not be used directly.
// This is only here so that the derive macro has something to work with.
impl SqliteIO for u8 {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl SqliteIO for u16 {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl SqliteIO for u32 {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl SqliteIO for i8 {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl SqliteIO for i16 {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl SqliteIO for i32 {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl SqliteIO for f32 {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl SqliteIO for String {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl SqliteIO for std::vec::Vec<u8> {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        vec!["".to_string()]
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        vec![self]
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        Ok((row.get::<usize, Self>(start_idx)?, start_idx + 1))
    }
}

impl<T: SqliteIO + Default + Copy, const N: usize> SqliteIO for [T; N] {
    fn sqlite_table() -> &'static str {
        ""
    }
    fn sqlite_columns() -> std::vec::Vec<String> {
        let t_columns = T::sqlite_columns();
        let mut columns = std::vec::Vec::<String>::new();
        for i in 0..N {
            for column in t_columns.iter() {
                columns.push(if column.is_empty() { format!("({})", i + 1) } else { format!("({}) {}", i + 1, column) });
            }
        }
        columns
    }
    fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
        let mut vals = std::vec::Vec::<&dyn rusqlite::ToSql>::new();
        for x in self {
            vals.extend_from_slice(&x.sqlite_to_tup());
        }
        vals
    }
    fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
        let mut idx = start_idx;
        let mut vals = [T::default(); N];
        for x in vals.iter_mut() {
            (*x, idx) = T::sqlite_row_read(row, idx)?;
        }
        Ok((vals, idx))
    }
}
