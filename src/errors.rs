//! Error types for `SQLite` load extension operations.

use alloc::string::String;
use thiserror::Error;

/// Errors that can occur when working with `SQLite` load extension functionality.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LoadExtensionError {
    /// Failed to enable extension loading before the load operation.
    #[error("Failed to enable load extension: {0}")]
    EnableFailed(String),

    /// Failed to load an extension from a shared library.
    #[error("Failed to load extension '{path}': {message}")]
    LoadFailed {
        /// Extension path that failed to load.
        path: String,
        /// Raw `SQLite` error message.
        message: String,
    },

    /// Failed to disable extension loading after the operation.
    #[error("Failed to disable load extension: {0}")]
    CleanupFailed(String),

    /// The provided extension path contains an interior null byte.
    #[error("Extension path contains an interior null byte")]
    InvalidPath,

    /// The provided entry point name contains an interior null byte.
    #[error("Entry point contains an interior null byte")]
    InvalidEntryPoint,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_enable_failed_display() {
        let err = LoadExtensionError::EnableFailed(String::from("not authorized"));
        assert_eq!(
            format!("{err}"),
            "Failed to enable load extension: not authorized"
        );
    }

    #[test]
    fn test_load_failed_display() {
        let err = LoadExtensionError::LoadFailed {
            path: String::from("mod_spatialite"),
            message: String::from("file not found"),
        };
        assert_eq!(
            format!("{err}"),
            "Failed to load extension 'mod_spatialite': file not found"
        );
    }

    #[test]
    fn test_cleanup_failed_display() {
        let err = LoadExtensionError::CleanupFailed(String::from("not authorized"));
        assert_eq!(
            format!("{err}"),
            "Failed to disable load extension: not authorized"
        );
    }

    #[test]
    fn test_invalid_path_display() {
        let err = LoadExtensionError::InvalidPath;
        assert_eq!(
            format!("{err}"),
            "Extension path contains an interior null byte"
        );
    }

    #[test]
    fn test_invalid_entry_point_display() {
        let err = LoadExtensionError::InvalidEntryPoint;
        assert_eq!(
            format!("{err}"),
            "Entry point contains an interior null byte"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_error_is_std_error() {
        fn assert_std_error<T: std::error::Error>() {}
        assert_std_error::<LoadExtensionError>();
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LoadExtensionError>();
    }
}
