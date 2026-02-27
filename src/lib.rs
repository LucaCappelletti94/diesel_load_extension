#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
compile_error!(
    "diesel_load_extension is native-only. On wasm32-unknown-unknown, use sqlite-wasm-rs and sqlite3_auto_extension directly."
);

mod errors;
mod ffi;

pub use errors::LoadExtensionError;

/// Extension trait for [`diesel::SqliteConnection`] providing `SQLite` load extension support.
#[diagnostic::on_unimplemented(
    message = "`SqliteLoadExtensionExt` is only implemented for `diesel::SqliteConnection`"
)]
pub trait SqliteLoadExtensionExt {
    /// Load a `SQLite` extension from a shared library file.
    ///
    /// This method enables extension loading, loads one extension, and then
    /// disables extension loading again.
    ///
    /// It wraps [`sqlite3_enable_load_extension`](https://www.sqlite.org/c3ref/enable_load_extension.html)
    /// and [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the shared library file containing the extension.
    /// * `entry_point` - Optional name of the extension's entry point function.
    ///   If `None`, `SQLite` uses a default entry point derived from the filename.
    ///
    /// # Errors
    ///
    /// Returns [`LoadExtensionError::EnableFailed`] if enabling extension loading fails.
    /// Returns [`LoadExtensionError::LoadFailed`] with path and `SQLite` error message if loading fails.
    /// Returns [`LoadExtensionError::InvalidPath`] if `path` contains a null byte.
    /// Returns [`LoadExtensionError::InvalidEntryPoint`] if `entry_point` contains a null byte.
    /// Returns [`LoadExtensionError::CleanupFailed`] if disabling extension loading fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use diesel::prelude::*;
    /// use diesel::SqliteConnection;
    /// use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};
    ///
    /// let mut conn = SqliteConnection::establish(":memory:").unwrap();
    /// let result = conn.load_extension("nonexistent_extension", None);
    /// assert!(matches!(result, Err(LoadExtensionError::LoadFailed { .. })));
    /// ```
    fn load_extension(
        &mut self,
        path: &str,
        entry_point: Option<&str>,
    ) -> Result<(), LoadExtensionError>;
}

// Native implementation — uses real FFI calls.
mod native_impl {
    use super::{ffi, LoadExtensionError, SqliteLoadExtensionExt};
    use alloc::ffi::CString;
    use alloc::format;
    use alloc::string::String;
    use core::ffi::{c_char, CStr};
    use core::ptr;
    use diesel::SqliteConnection;

    #[allow(unsafe_code)]
    fn sqlite_error_message(raw: *mut ffi::sqlite3) -> String {
        // SAFETY: `raw` is a valid SQLite connection pointer from Diesel.
        let err_ptr = unsafe { ffi::sqlite3_errmsg(raw) };
        // SAFETY: `sqlite3_errmsg` returns a valid, null-terminated C string.
        unsafe { CStr::from_ptr(err_ptr).to_string_lossy().into_owned() }
    }

    #[allow(unsafe_code)]
    fn toggle_extension_loading(raw: *mut ffi::sqlite3, enabled: bool) -> Result<(), String> {
        // SAFETY: `raw` is a valid SQLite connection pointer from Diesel.
        let rc = unsafe { ffi::sqlite3_enable_load_extension(raw, i32::from(enabled)) };
        if rc == ffi::SQLITE_OK {
            Ok(())
        } else {
            Err(sqlite_error_message(raw))
        }
    }

    #[allow(unsafe_code)]
    fn load_extension_once(
        raw: *mut ffi::sqlite3,
        c_path: &CString,
        c_entry: Option<&CString>,
    ) -> Result<(), String> {
        let mut err_msg: *mut c_char = ptr::null_mut();
        let entry_ptr = c_entry.map_or(ptr::null(), |c| c.as_ptr());

        // SAFETY:
        // - `raw` is a valid SQLite connection pointer.
        // - `c_path` and `c_entry` are valid C strings for the duration of the call.
        // - `err_msg` is a valid out-pointer for SQLite to populate.
        let rc = unsafe {
            ffi::sqlite3_load_extension(raw, c_path.as_ptr(), entry_ptr, &raw mut err_msg)
        };

        if rc == ffi::SQLITE_OK {
            return Ok(());
        }

        if err_msg.is_null() {
            return Err(sqlite_error_message(raw));
        }

        // SAFETY: non-null `err_msg` points to a valid null-terminated SQLite error string.
        let msg = unsafe { CStr::from_ptr(err_msg).to_string_lossy().into_owned() };
        // SAFETY: non-null `err_msg` is owned by SQLite and must be freed with `sqlite3_free`.
        unsafe { ffi::sqlite3_free(err_msg.cast()) };
        Err(msg)
    }

    #[allow(unsafe_code)]
    impl SqliteLoadExtensionExt for SqliteConnection {
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        fn load_extension(
            &mut self,
            path: &str,
            entry_point: Option<&str>,
        ) -> Result<(), LoadExtensionError> {
            let c_path = CString::new(path).map_err(|_| LoadExtensionError::InvalidPath)?;
            let c_entry = entry_point
                .map(|ep| CString::new(ep).map_err(|_| LoadExtensionError::InvalidEntryPoint))
                .transpose()?;
            let path = String::from(path);

            // SAFETY: `with_raw_connection` provides a valid SQLite pointer for the closure.
            unsafe {
                self.with_raw_connection(|raw| {
                    toggle_extension_loading(raw, true)
                        .map_err(LoadExtensionError::EnableFailed)?;
                    let load_result = load_extension_once(raw, &c_path, c_entry.as_ref());
                    let disable_result = toggle_extension_loading(raw, false);

                    match (load_result, disable_result) {
                        (Ok(()), Ok(())) => Ok(()),
                        (Err(load_message), Ok(())) => Err(LoadExtensionError::LoadFailed {
                            path: path.clone(),
                            message: load_message,
                        }),
                        (Ok(()), Err(cleanup_message)) => {
                            Err(LoadExtensionError::CleanupFailed(cleanup_message))
                        }
                        (Err(load_message), Err(cleanup_message)) => {
                            Err(LoadExtensionError::CleanupFailed(format!(
                                "{cleanup_message}; load also failed for '{path}': {load_message}"
                            )))
                        }
                    }
                })
            }
        }
    }

    #[cfg(test)]
    #[allow(clippy::redundant_pub_crate)]
    #[allow(unsafe_code)]
    pub(super) fn raw_load_extension_without_toggle(
        conn: &mut SqliteConnection,
        path: &str,
        entry_point: Option<&str>,
    ) -> Result<(), String> {
        let c_path = CString::new(path).map_err(|_| "invalid test path".to_owned())?;
        let c_entry = entry_point
            .map(|ep| CString::new(ep).map_err(|_| "invalid test entry point".to_owned()))
            .transpose()?;

        // SAFETY: `with_raw_connection` provides a valid SQLite pointer for the closure.
        unsafe {
            conn.with_raw_connection(|raw| load_extension_once(raw, &c_path, c_entry.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{native_impl, LoadExtensionError, SqliteLoadExtensionExt};
    use diesel::prelude::*;

    fn create_connection() -> SqliteConnection {
        SqliteConnection::establish(":memory:").expect("Failed to create in-memory connection")
    }

    #[test]
    fn test_load_nonexistent_extension() {
        let mut conn = create_connection();
        let result = conn.load_extension("/nonexistent/path/to/extension.so", None);
        assert!(matches!(result, Err(LoadExtensionError::LoadFailed { .. })));
    }

    #[test]
    fn test_load_extension_disables_after_failure() {
        let mut conn = create_connection();
        let _ = conn.load_extension("/nonexistent/extension.so", None);

        let result =
            native_impl::raw_load_extension_without_toggle(&mut conn, "some_extension", None);
        assert!(result.is_err());
        assert!(
            !result.unwrap_err().is_empty(),
            "Expected non-empty error message"
        );
    }

    #[test]
    fn test_load_extension_with_entry_point() {
        let mut conn = create_connection();
        let result = conn.load_extension("/nonexistent/extension.so", Some("my_init"));
        assert!(matches!(result, Err(LoadExtensionError::LoadFailed { .. })));
    }

    #[test]
    fn test_invalid_path_null_byte() {
        let mut conn = create_connection();
        let result = conn.load_extension("path\0with_null", None);
        assert!(matches!(
            result.unwrap_err(),
            LoadExtensionError::InvalidPath
        ));
    }

    #[test]
    fn test_invalid_entry_point_null_byte() {
        let mut conn = create_connection();
        let result = conn.load_extension("some_extension", Some("entry\0point"));
        assert!(matches!(
            result.unwrap_err(),
            LoadExtensionError::InvalidEntryPoint
        ));
    }
}
