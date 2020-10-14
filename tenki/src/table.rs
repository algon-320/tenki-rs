#![allow(dead_code)]

use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Incompatible,
}
pub type Result<T> = std::result::Result<T, Error>;

pub enum Cell {
    Left(Box<dyn Display>),
    Center(Box<dyn Display>),
    Right(Box<dyn Display>),
    VarticalBorder,
    HorizontalBorder,
    Empty,
}
impl Cell {
    pub fn new_left<T: Display + 'static>(value: T) -> Self {
        Cell::Left(Box::new(value))
    }
    pub fn new_center<T: Display + 'static>(value: T) -> Self {
        Cell::Center(Box::new(value))
    }
    pub fn new_right<T: Display + 'static>(value: T) -> Self {
        Cell::Right(Box::new(value))
    }

    pub fn fmt(&self, width: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn calc_pad(item: &dyn Display, width: usize) -> usize {
            use ucd::Codepoint; // for east_asian_width
            let (char_count, display_width) = item.to_string().chars().fold((0, 0), |mut a, ch| {
                a.0 += 1;
                a.1 += match ch.east_asian_width() {
                    ucd::EastAsianWidth::Narrow => 1,
                    ucd::EastAsianWidth::Wide => 2,
                    ucd::EastAsianWidth::Neutral => 1,
                    ucd::EastAsianWidth::Ambiguous => 1,
                    ucd::EastAsianWidth::FullWidth => 2,
                    ucd::EastAsianWidth::HalfWidth => 1,
                };
                a
            });
            width - display_width + char_count
        }
        match self {
            Cell::Left(value) => write!(f, "{:<1$}", value, calc_pad(value, width)),
            Cell::Center(value) => write!(f, "{:^1$}", value, calc_pad(value, width)),
            Cell::Right(value) => write!(f, "{:>1$}", value, calc_pad(value, width)),
            Cell::VarticalBorder => write!(f, "{:>1$}", "|", width),
            Cell::HorizontalBorder => write!(f, "{:=<1$}", "", width),
            Cell::Empty => write!(f, "{:1$}", "", width),
        }
    }
}

pub struct Table {
    title: String,
    column_count: usize,
    matrix: Vec<Vec<Cell>>,
}
impl Table {
    pub fn empty(title: impl Into<String>, column_count: usize) -> Self {
        Self {
            title: title.into(),
            column_count,
            matrix: Vec::new(),
        }
    }
    pub fn add_row(&mut self, row: Vec<Cell>) -> Result<()> {
        if self.column_count != row.len() {
            Err(Error::Incompatible)
        } else {
            self.matrix.push(row);
            Ok(())
        }
    }
    pub fn add_horizontal_border(&mut self) {
        let mut row = Vec::new();
        for _ in 0..self.column_count {
            row.push(Cell::HorizontalBorder);
        }
        self.add_row(row).unwrap();
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)?;
        for row in self.matrix.iter() {
            writeln!(f)?;
            for (i, val) in row.iter().enumerate() {
                // FIXME
                if i == 0 {
                    val.fmt(1, f)?;
                } else if i == 1 {
                    val.fmt(16, f)?;
                } else if i == 2 {
                    val.fmt(2, f)?;
                } else if i == self.column_count - 1 {
                    val.fmt(2, f)?;
                } else {
                    val.fmt(7, f)?;
                }
            }
        }
        Ok(())
    }
}
