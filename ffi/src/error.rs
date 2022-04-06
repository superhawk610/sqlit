use std::cell::RefCell;
use thiserror::Error;

thread_local! {
    static LAST_ERR: RefCell<Option<String>> = RefCell::new(None);
}

pub fn set_last_err<E: std::error::Error>(err: E) {
    LAST_ERR.with(|prev| *prev.borrow_mut() = Some(format!("{}", err)));
}

pub fn take_last_err() -> Option<String> {
    LAST_ERR.with(|err| err.borrow_mut().take())
}

#[derive(Error, Debug)]
pub enum SQLitError {
    // database
    #[error("unable to connect to database: {0:?}")]
    ConnectionFailure(#[from] std::io::Error),
    // query parsing
    #[error("invalid query: {0}")]
    InvalidQuery(String),
    // query execution
    #[error("table `{0}` already exists")]
    TableAlreadyExists(String),
    #[error("unable to insert: {0}")]
    InvalidInsert(String),
}
