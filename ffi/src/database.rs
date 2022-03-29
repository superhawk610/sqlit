use crate::error::SQLitError;
use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug)]
pub struct Database {
    tables: HashMap<String, Table>,
    // this will eventually be an actual File
    file: String,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub rows: Rows,
    // index in `columns`
    pub primary_key: usize,
    /// default: false
    pub autoincrement: bool,
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub t: ColumnType,
    /// default: None
    pub default: Option<Value>,
    /// default: true
    pub allow_null: bool,
    // this should be moved into a separate constraint eventually
    /// default: false
    pub unique: bool,
}

#[derive(Debug)]
pub enum ColumnType {
    Integer,
    Text,
    Real,
    Blob,
}

pub struct Rows(Vec<Vec<Value>>);

#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Text(String),
    Real(f64),
    Blob(Vec<u8>),
}

impl std::fmt::Debug for Rows {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rows(count={})", self.0.len())
    }
}

pub enum CreateTableResult {
    Created,
    Skipped,
}

impl Database {
    pub fn open(file: String) -> Self {
        Database {
            tables: HashMap::new(),
            file,
        }
    }

    pub fn create_table(
        &mut self,
        table: Table,
        if_not_exists: bool,
    ) -> Result<CreateTableResult, SQLitError> {
        match self.tables.entry(table.name.clone()) {
            Entry::Occupied(_) if if_not_exists => Ok(CreateTableResult::Skipped),
            Entry::Occupied(_) => Err(SQLitError::TableAlreadyExists(table.name.clone())),
            Entry::Vacant(entry) => {
                entry.insert(table);
                Ok(CreateTableResult::Created)
            }
        }
    }
}

impl Table {
    pub fn new(
        name: String,
        columns: Vec<Column>,
        primary_key: usize,
        autoincrement: bool,
    ) -> Self {
        Self {
            name,
            columns,
            rows: Rows(Vec::new()),
            primary_key,
            autoincrement,
        }
    }
}
