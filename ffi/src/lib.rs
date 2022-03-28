mod parser;
mod query;

use query::Query;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn sqlit_parse_query(input: *const c_char) -> *mut Query {
    let input = unsafe { CStr::from_ptr(input) }
        .to_str()
        .expect("valid UTF-8");
    Box::into_raw(Box::new(Query::parse(input)))
}

#[no_mangle]
pub extern "C" fn sqlit_debug_query(query: *mut Query) -> *mut c_char {
    let query = unsafe { Box::from_raw(query) };
    let s = CString::new(format!("{:?}", query)).unwrap();
    std::mem::forget(query);
    CString::into_raw(s)
}

#[no_mangle]
pub extern "C" fn sqlit_free_query(query: *mut Query) {
    unsafe {
        if query.is_null() {
            return;
        }
        drop(Box::from_raw(query));
    }
}

#[no_mangle]
pub extern "C" fn sqlit_free_string(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        drop(CString::from_raw(s));
    }
}
