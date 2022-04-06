use crate::error::SQLitError;
use std::collections::{hash_map::Entry, HashMap, HashSet};

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

pub struct Rows(Vec<Vec<Option<Value>>>);

pub struct RowView<'a>(Vec<Vec<&'a Option<Value>>>);

#[derive(Debug, Clone, PartialEq)]
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

    pub fn insert<'a, V: 'a>(&mut self, values: V) -> Result<(), SQLitError>
    where
        V: IntoIterator<Item = (&'a str, Value)>,
    {
        let mut values: HashMap<&'a str, Value> = HashMap::from_iter(values.into_iter());
        let mut row = Vec::with_capacity(self.columns.len());

        for column in &self.columns {
            let value = values
                .remove(&column.name[..])
                .or_else(|| column.default.clone());
            row.push(value);

            // TODO: column type must match
            // TODO: enforce unique constraint
            // TODO: autoincrement
        }

        self.rows.0.push(row);

        Ok(())
    }

    pub fn select<'c, 't, C: 'c>(&'t self, columns: C) -> RowView<'t>
    where
        C: IntoIterator<Item = &'c str>,
    {
        let columns: HashSet<&'c str> = HashSet::from_iter(columns.into_iter());

        RowView(
            self.rows
                .0
                .iter()
                .map(|row| {
                    self.columns
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| columns.contains(&c.name[..]))
                        .map(|(i, _)| {
                            // Safety: Each row in `self.rows` is guaranteed to have the same length
                            // as `self.columns`, as an invariant.
                            unsafe { row.get_unchecked(i) }
                        })
                        .collect()
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_insert_and_select() {
        let columns = vec![Column {
            name: "id".to_string(),
            t: ColumnType::Integer,
            default: None,
            allow_null: false,
            unique: true,
        }];
        let mut table = Table::new("users".to_string(), columns, 0, true);

        // inserting a single row
        table.insert(vec![("id", Value::Integer(1))]).unwrap();
        assert_eq!(table.rows.0.len(), 1);

        // querying for a single row
        let rows = table.select(vec!["id"]);
        assert_eq!(rows.0.len(), 1);
        assert_eq!(rows.0[0].len(), 1);
        assert_eq!(rows.0[0][0], &Some(Value::Integer(1)));
    }
}
