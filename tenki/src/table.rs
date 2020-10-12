use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Incompatible,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ColumnLayout {
    Left,
    Center,
    Right,
}
#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub layout: ColumnLayout,
}

#[derive(Debug)]
pub enum Row<T: Display> {
    Border,
    Item(Vec<T>),
}

#[derive(Debug)]
pub struct Table<T: Display> {
    title: String,
    columns: Vec<Column>,
    rows: Vec<Row<T>>,
}
impl<T: Display> Table<T> {
    pub fn empty(title: impl Into<String>, columns: Vec<Column>) -> Self {
        Self {
            title: title.into(),
            columns,
            rows: Vec::new(),
        }
    }
    pub fn with_rows(title: impl Into<String>, columns: Vec<Column>, rows: Vec<Row<T>>) -> Self {
        Self {
            title: title.into(),
            columns,
            rows,
        }
    }
    pub fn add_row(&mut self, row: Row<T>) -> Result<()> {
        if let Row::Item(item) = &row {
            if self.columns.len() != item.len() {
                return Err(Error::Incompatible);
            }
        }
        self.rows.push(row);
        Ok(())
    }
}
