//! Hand-written FFI bindings for `SQLite` load extension functions.

use core::ffi::{c_char, c_int};
pub use libsqlite3_sys::{sqlite3, sqlite3_free, SQLITE_OK};

// On native targets, declare the load extension FFI functions as hand-written
// extern "C" bindings that link to the SQLite C library.
#[allow(unsafe_code)]
extern "C" {
    pub fn sqlite3_enable_load_extension(db: *mut sqlite3, onoff: c_int) -> c_int;

    pub fn sqlite3_load_extension(
        db: *mut sqlite3,
        file: *const c_char,
        entry_point: *const c_char,
        err_msg: *mut *mut c_char,
    ) -> c_int;

    pub fn sqlite3_errmsg(db: *mut sqlite3) -> *const c_char;
}
