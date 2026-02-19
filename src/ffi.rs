//! Hand-written FFI bindings for `SQLite` load extension functions.

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub use libsqlite3_sys::{sqlite3, sqlite3_free, SQLITE_OK};

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub use sqlite_wasm_rs::{sqlite3, sqlite3_free, SQLITE_OK};

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use std::os::raw::{c_char, c_int};

// On native targets, declare the load extension FFI functions as hand-written
// extern "C" bindings that link to the SQLite C library.
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
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

// On WASM targets, import the stub functions from sqlite-wasm-rs.
// These always return SQLITE_ERROR since dynamic library loading is not
// supported in WebAssembly.
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub use sqlite_wasm_rs::{sqlite3_enable_load_extension, sqlite3_load_extension};
