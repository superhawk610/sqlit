mod database;
mod error;
mod parser;
mod query;

use database::Database;
use error::{set_last_err, take_last_err};
use query::Query;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Retrieve and clear the most recent error, if any.
#[no_mangle]
pub extern "C" fn sqlit_error_last() -> *mut c_char {
    match take_last_err() {
        None => std::ptr::null_mut(),
        Some(err) => CString::into_raw(CString::new(err).unwrap()),
    }
}

/// Returns NULL on failure (make sure to check `sqlit_error_last`).
#[no_mangle]
pub unsafe extern "C" fn sqlit_db_open(file: *const c_char) -> *mut Database {
    let file = CStr::from_ptr(file).to_str().expect("valid UTF-8");
    Box::into_raw(Box::new(Database::open(file.to_string())))
}

#[no_mangle]
pub unsafe extern "C" fn sqlit_db_debug(db: *mut Database) -> *mut c_char {
    CString::into_raw(CString::new(format!("{:?}", &*db)).unwrap())
}

#[no_mangle]
pub extern "C" fn sqlit_db_close(db: *mut Database) {
    if db.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(db));
    }
}

/// Returns NULL on failure (make sure to check `sqlit_error_last`).
#[no_mangle]
pub unsafe extern "C" fn sqlit_query_parse(input: *const c_char) -> *mut Query {
    let input = CStr::from_ptr(input).to_str().expect("valid UTF-8");
    match Query::parse(input) {
        Ok(query) => Box::into_raw(Box::new(query)),
        Err(err) => {
            set_last_err(err);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlit_query_debug(query: *mut Query) -> *mut c_char {
    CString::into_raw(CString::new(format!("{:?}", &*query)).unwrap())
}

#[no_mangle]
pub extern "C" fn sqlit_query_free(query: *mut Query) {
    if query.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(query));
    }
}

#[no_mangle]
pub extern "C" fn sqlit_string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(s));
    }
}
