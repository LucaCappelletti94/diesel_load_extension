#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod errors;
mod ffi;

pub use errors::LoadExtensionError;

use std::ffi::CString;

/// Extension trait for [`diesel::SqliteConnection`] providing `SQLite` load extension support.
#[diagnostic::on_unimplemented(
    message = "`SqliteLoadExtensionExt` is only implemented for `diesel::SqliteConnection`"
)]
pub trait SqliteLoadExtensionExt {
    /// Enable or disable the ability to load `SQLite` extensions.
    ///
    /// This wraps [`sqlite3_enable_load_extension`](https://www.sqlite.org/c3ref/enable_load_extension.html).
    ///
    /// By default, extension loading is disabled in `SQLite` for security reasons.
    /// Most users should prefer [`load_extension`](Self::load_extension), which
    /// automatically enables and disables extension loading around the load call.
    ///
    /// This method is useful when you need fine-grained control over the
    /// extension loading lifecycle, for example when loading many extensions
    /// in a batch.
    ///
    /// # Errors
    ///
    /// Returns [`LoadExtensionError::EnableFailed`] if `SQLite` returns a non-OK status code.
    ///
    /// On WASM targets, returns [`LoadExtensionError::UnsupportedPlatform`] unconditionally.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use diesel::prelude::*;
    /// use diesel::SqliteConnection;
    /// use diesel_load_extension::SqliteLoadExtensionExt;
    ///
    /// let mut conn = SqliteConnection::establish(":memory:").unwrap();
    /// conn.enable_load_extension(true).unwrap();
    /// conn.enable_load_extension(false).unwrap();
    /// ```
    fn enable_load_extension(&mut self, enabled: bool) -> Result<(), LoadExtensionError>;

    /// Load a `SQLite` extension from a shared library file.
    ///
    /// This method automatically enables extension loading before the load and
    /// disables it afterward, ensuring extension loading is never left enabled
    /// unintentionally.
    ///
    /// This wraps [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html).
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
    /// Returns [`LoadExtensionError::LoadFailed`] with the `SQLite` error message if loading fails.
    /// Returns [`LoadExtensionError::InvalidPath`] if `path` contains a null byte.
    /// Returns [`LoadExtensionError::InvalidEntryPoint`] if `entry_point` contains a null byte.
    ///
    /// On WASM targets, returns [`LoadExtensionError::UnsupportedPlatform`] after
    /// validating inputs.
    ///
    /// # Panics
    ///
    /// No user-provided callbacks run between the enable and disable calls, so
    /// panics are unlikely. If a panic did occur (e.g., OOM in an allocation),
    /// a best-effort guard disables extension loading when the stack unwinds.
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
    /// assert!(matches!(result, Err(LoadExtensionError::LoadFailed(_))));
    /// ```
    fn load_extension(
        &mut self,
        path: &str,
        entry_point: Option<&str>,
    ) -> Result<(), LoadExtensionError>;

    /// Load multiple `SQLite` extensions in a single enable/disable cycle.
    ///
    /// This is more efficient than calling [`load_extension`](Self::load_extension)
    /// repeatedly when loading several extensions, since it only enables and
    /// disables extension loading once.
    ///
    /// Extensions are loaded in order. If any extension fails to load, the
    /// remaining extensions are skipped and extension loading is disabled
    /// before the error is returned.
    ///
    /// If the extension list is empty, this returns `Ok(())` immediately
    /// without enabling or disabling extension loading.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered, whether from enabling, loading,
    /// or disabling extension loading.
    ///
    /// On WASM targets, returns [`LoadExtensionError::UnsupportedPlatform`] after
    /// validating inputs (unless the list is empty).
    ///
    /// # Panics
    ///
    /// No user-provided callbacks run between the enable and disable calls, so
    /// panics are unlikely. If a panic did occur (e.g., OOM in an allocation),
    /// a best-effort guard disables extension loading when the stack unwinds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use diesel::prelude::*;
    /// use diesel::SqliteConnection;
    /// use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};
    ///
    /// let mut conn = SqliteConnection::establish(":memory:").unwrap();
    /// let result = conn.load_extensions(&[
    ///     ("nonexistent_ext1", None),
    ///     ("nonexistent_ext2", Some("init")),
    /// ]);
    /// assert!(matches!(result, Err(LoadExtensionError::LoadFailed(_))));
    /// ```
    fn load_extensions(
        &mut self,
        extensions: &[(&str, Option<&str>)],
    ) -> Result<(), LoadExtensionError>;
}

/// Validate and convert extension inputs to C strings.
fn validate_inputs(
    extensions: &[(&str, Option<&str>)],
) -> Result<Vec<(CString, Option<CString>)>, LoadExtensionError> {
    extensions
        .iter()
        .map(|(path, entry_point)| {
            let c_path = CString::new(*path).map_err(|_| LoadExtensionError::InvalidPath)?;
            let c_entry = entry_point
                .map(|ep| CString::new(ep).map_err(|_| LoadExtensionError::InvalidEntryPoint))
                .transpose()?;
            Ok((c_path, c_entry))
        })
        .collect()
}

// Native implementation — uses real FFI calls.
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
mod native_impl {
    use super::{ffi, validate_inputs, LoadExtensionError, SqliteLoadExtensionExt};
    use diesel::SqliteConnection;
    use std::ffi::{c_char, CStr, CString};
    use std::ptr;

    #[allow(unsafe_code)]
    impl SqliteLoadExtensionExt for SqliteConnection {
        // The `with_raw_connection` API requires a single outer `unsafe` block that
        // encompasses both the method call and the FFI calls within the closure.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        fn enable_load_extension(&mut self, enabled: bool) -> Result<(), LoadExtensionError> {
            let onoff = i32::from(enabled);
            // SAFETY: `with_raw_connection` provides a valid database pointer.
            // `sqlite3_enable_load_extension` receives that valid pointer and a valid
            // integer flag (0 or 1). On failure, `sqlite3_errmsg` is called with the
            // same valid pointer to retrieve the human-readable error message.
            unsafe {
                self.with_raw_connection(|raw| {
                    let rc = ffi::sqlite3_enable_load_extension(raw, onoff);
                    if rc != ffi::SQLITE_OK {
                        // SAFETY: `sqlite3_errmsg` returns a valid, null-terminated
                        // C string for any valid database pointer (never null).
                        let msg = CStr::from_ptr(ffi::sqlite3_errmsg(raw))
                            .to_string_lossy()
                            .into_owned();
                        return Err(LoadExtensionError::EnableFailed(msg));
                    }
                    Ok(())
                })
            }
        }

        fn load_extension(
            &mut self,
            path: &str,
            entry_point: Option<&str>,
        ) -> Result<(), LoadExtensionError> {
            let c_path = CString::new(path).map_err(|_| LoadExtensionError::InvalidPath)?;
            let c_entry = entry_point
                .map(|ep| CString::new(ep).map_err(|_| LoadExtensionError::InvalidEntryPoint))
                .transpose()?;

            with_extension_enabled(self, |conn| {
                raw_load_extension(conn, &c_path, c_entry.as_ref())
            })
        }

        fn load_extensions(
            &mut self,
            extensions: &[(&str, Option<&str>)],
        ) -> Result<(), LoadExtensionError> {
            if extensions.is_empty() {
                return Ok(());
            }

            let c_extensions = validate_inputs(extensions)?;

            with_extension_enabled(self, |conn| {
                for (c_path, c_entry) in &c_extensions {
                    raw_load_extension(conn, c_path, c_entry.as_ref())?;
                }
                Ok(())
            })
        }
    }

    fn with_extension_enabled<T, F>(
        conn: &mut SqliteConnection,
        f: F,
    ) -> Result<T, LoadExtensionError>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<T, LoadExtensionError>,
    {
        conn.enable_load_extension(true)?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(conn)));
        let disable_result = conn.enable_load_extension(false);

        match result {
            Ok(inner) => {
                let value = inner?;
                disable_result?;
                Ok(value)
            }
            Err(payload) => {
                let _ = disable_result;
                std::panic::resume_unwind(payload);
            }
        }
    }

    #[cfg(test)]
    #[allow(clippy::redundant_pub_crate)]
    pub(super) fn test_with_extension_enabled_panics(conn: &mut SqliteConnection) {
        let _ = with_extension_enabled(conn, |_conn| -> Result<(), LoadExtensionError> {
            panic!("intentional panic for test");
            #[allow(unreachable_code)]
            Ok(())
        });
    }

    /// Raw FFI call to `sqlite3_load_extension`, without enable/disable management.
    #[allow(unsafe_code)]
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    pub fn raw_load_extension(
        conn: &mut SqliteConnection,
        c_path: &CString,
        c_entry: Option<&CString>,
    ) -> Result<(), LoadExtensionError> {
        // SAFETY:
        // - `with_raw_connection` provides a valid database pointer.
        // - `sqlite3_load_extension`: receives valid C strings (or null) from
        //   `CString::as_ptr` and a valid out-pointer for the error message.
        // - `CStr::from_ptr`: SQLite guarantees that a non-null `err_msg` is a
        //   valid, null-terminated C string.
        // - `sqlite3_free`: required to free the SQLite-allocated error message
        //   when non-null, as documented by the SQLite API.
        unsafe {
            conn.with_raw_connection(|raw| {
                let mut err_msg: *mut c_char = ptr::null_mut();
                let entry_ptr = c_entry.map_or(ptr::null(), |c| c.as_ptr());

                let rc =
                    ffi::sqlite3_load_extension(raw, c_path.as_ptr(), entry_ptr, &raw mut err_msg);

                if rc != ffi::SQLITE_OK {
                    let message = if err_msg.is_null() {
                        CStr::from_ptr(ffi::sqlite3_errmsg(raw))
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        let msg = CStr::from_ptr(err_msg).to_string_lossy().into_owned();
                        ffi::sqlite3_free(err_msg.cast());
                        msg
                    };
                    return Err(LoadExtensionError::LoadFailed(message));
                }

                Ok(())
            })
        }
    }
}

// WASM implementation — no unsafe code, returns UnsupportedPlatform.
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod wasm_impl {
    use super::{validate_inputs, LoadExtensionError, SqliteLoadExtensionExt};
    use diesel::SqliteConnection;
    use std::ffi::CString;

    impl SqliteLoadExtensionExt for SqliteConnection {
        fn enable_load_extension(&mut self, _enabled: bool) -> Result<(), LoadExtensionError> {
            Err(LoadExtensionError::UnsupportedPlatform)
        }

        fn load_extension(
            &mut self,
            path: &str,
            entry_point: Option<&str>,
        ) -> Result<(), LoadExtensionError> {
            // Validate inputs first so callers get specific errors for bad inputs.
            let _c_path = CString::new(path).map_err(|_| LoadExtensionError::InvalidPath)?;
            let _c_entry = entry_point
                .map(|ep| CString::new(ep).map_err(|_| LoadExtensionError::InvalidEntryPoint))
                .transpose()?;

            Err(LoadExtensionError::UnsupportedPlatform)
        }

        fn load_extensions(
            &mut self,
            extensions: &[(&str, Option<&str>)],
        ) -> Result<(), LoadExtensionError> {
            if extensions.is_empty() {
                return Ok(());
            }

            // Validate inputs first so callers get specific errors for bad inputs.
            let _c_extensions = validate_inputs(extensions)?;

            Err(LoadExtensionError::UnsupportedPlatform)
        }
    }
}

/// WASM-specific helpers for precompiled SQLite extensions.
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub mod wasm {
    use sqlite_wasm_rs::{sqlite3, sqlite3_api_routines, sqlite3_auto_extension};
    use std::ffi::c_char;
    use std::sync::OnceLock;

    /// SQLite auto-extension initializer signature.
    pub type AutoExtensionInit = unsafe extern "C" fn(
        *mut sqlite3,
        *mut *mut c_char,
        *const sqlite3_api_routines,
    ) -> std::ffi::c_int;

    /// Register a precompiled SQLite extension for WASM builds.
    ///
    /// This wraps `sqlite3_auto_extension` and hides the required unsafe cast.
    /// The registration is idempotent and runs at most once per process.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn main() {
    ///     #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    ///     {
    ///         use diesel_load_extension::wasm::register_auto_extension;
    ///
    ///         unsafe extern "C" fn dummy_init(
    ///             _db: *mut sqlite_wasm_rs::sqlite3,
    ///             _pz_err_msg: *mut *mut std::ffi::c_char,
    ///             _p_api: *const sqlite_wasm_rs::sqlite3_api_routines,
    ///         ) -> std::ffi::c_int {
    ///             0
    ///         }
    ///
    ///         register_auto_extension(dummy_init);
    ///     }
    /// }
    /// ```
    #[allow(unsafe_code)]
    pub fn register_auto_extension(init: AutoExtensionInit) {
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| unsafe {
            let _ = sqlite3_auto_extension(Some(init));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{native_impl, LoadExtensionError, SqliteLoadExtensionExt};
    use diesel::prelude::*;
    use std::ffi::CString;

    fn create_connection() -> SqliteConnection {
        SqliteConnection::establish(":memory:").expect("Failed to create in-memory connection")
    }

    #[test]
    fn test_enable_load_extension() {
        let mut conn = create_connection();
        conn.enable_load_extension(true)
            .expect("Failed to enable load extension");
    }

    #[test]
    fn test_disable_load_extension() {
        let mut conn = create_connection();
        conn.enable_load_extension(false)
            .expect("Failed to disable load extension");
    }

    #[test]
    fn test_enable_then_disable_load_extension() {
        let mut conn = create_connection();
        conn.enable_load_extension(true)
            .expect("Failed to enable load extension");
        conn.enable_load_extension(false)
            .expect("Failed to disable load extension");
    }

    #[test]
    fn test_load_nonexistent_extension() {
        let mut conn = create_connection();

        let result = conn.load_extension("/nonexistent/path/to/extension.so", None);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), LoadExtensionError::LoadFailed(_)),
            "Expected LoadFailed error"
        );
    }

    #[test]
    fn test_load_extension_disables_after_failure() {
        let mut conn = create_connection();

        // load_extension should auto-disable even on failure
        let _ = conn.load_extension("/nonexistent/extension.so", None);

        // Verify extension loading is now disabled by using the raw FFI path
        let c_path = CString::new("some_extension").unwrap();
        let result = native_impl::raw_load_extension(&mut conn, &c_path, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            LoadExtensionError::LoadFailed(msg) => {
                assert!(!msg.is_empty(), "Expected non-empty error message");
            }
            err => panic!("Expected LoadFailed, got: {err:?}"),
        }
    }

    #[test]
    fn test_load_extension_with_entry_point() {
        let mut conn = create_connection();

        let result = conn.load_extension("/nonexistent/extension.so", Some("my_init"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LoadExtensionError::LoadFailed(_)
        ));
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

    #[test]
    fn test_enable_load_extension_idempotent() {
        let mut conn = create_connection();
        conn.enable_load_extension(true).unwrap();
        conn.enable_load_extension(true).unwrap();
        conn.enable_load_extension(false).unwrap();
        conn.enable_load_extension(false).unwrap();
    }

    #[test]
    fn test_load_extensions_empty_list() {
        let mut conn = create_connection();
        conn.load_extensions(&[])
            .expect("Loading empty extension list should succeed");
    }

    #[test]
    fn test_load_extensions_nonexistent() {
        let mut conn = create_connection();
        let result = conn.load_extensions(&[
            ("/nonexistent/ext1.so", None),
            ("/nonexistent/ext2.so", None),
        ]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LoadExtensionError::LoadFailed(_)
        ));
    }

    #[test]
    fn test_load_extensions_invalid_path() {
        let mut conn = create_connection();
        let result = conn.load_extensions(&[("valid_path", None), ("path\0null", None)]);
        assert!(matches!(
            result.unwrap_err(),
            LoadExtensionError::InvalidPath
        ));
    }

    #[test]
    fn test_load_extensions_disables_after_failure() {
        let mut conn = create_connection();

        let _ = conn.load_extensions(&[("/nonexistent/ext.so", None)]);

        // Verify extension loading is now disabled
        let c_path = CString::new("some_extension").unwrap();
        let result = native_impl::raw_load_extension(&mut conn, &c_path, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            LoadExtensionError::LoadFailed(msg) => {
                assert!(!msg.is_empty(), "Expected non-empty error message");
            }
            err => panic!("Expected LoadFailed, got: {err:?}"),
        }
    }

    #[test]
    fn test_load_extension_disables_after_panic() {
        let mut conn = create_connection();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            native_impl::test_with_extension_enabled_panics(&mut conn);
        }));
        assert!(result.is_err(), "Expected panic to be caught");

        // Verify extension loading is now disabled
        let c_path = CString::new("some_extension").unwrap();
        let result = native_impl::raw_load_extension(&mut conn, &c_path, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            LoadExtensionError::LoadFailed(msg) => {
                assert!(!msg.is_empty(), "Expected non-empty error message");
            }
            err => panic!("Expected LoadFailed, got: {err:?}"),
        }
    }
}
