//! Error types for `SQLite` load extension operations.

use thiserror::Error;

/// Errors that can occur when working with `SQLite` load extension functionality.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LoadExtensionError {
    /// Failed to enable or disable load extension support.
    #[error("Failed to enable/disable load extension: {0}")]
    EnableFailed(String),

    /// Failed to load an extension from a shared library.
    #[error("Failed to load extension: {0}")]
    LoadFailed(String),

    /// The provided extension path contains an interior null byte.
    #[error("Extension path contains an interior null byte")]
    InvalidPath,

    /// The provided entry point name contains an interior null byte.
    #[error("Entry point contains an interior null byte")]
    InvalidEntryPoint,

    /// Extension loading is not supported on this platform.
    #[error("Extension loading is not supported on this platform (WASM)")]
    UnsupportedPlatform,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_failed_display() {
        let err = LoadExtensionError::EnableFailed("not authorized".to_string());
        assert_eq!(
            err.to_string(),
            "Failed to enable/disable load extension: not authorized"
        );
    }

    #[test]
    fn test_load_failed_display() {
        let err = LoadExtensionError::LoadFailed("file not found".to_string());
        assert_eq!(err.to_string(), "Failed to load extension: file not found");
    }

    #[test]
    fn test_invalid_path_display() {
        let err = LoadExtensionError::InvalidPath;
        assert_eq!(
            err.to_string(),
            "Extension path contains an interior null byte"
        );
    }

    #[test]
    fn test_invalid_entry_point_display() {
        let err = LoadExtensionError::InvalidEntryPoint;
        assert_eq!(
            err.to_string(),
            "Entry point contains an interior null byte"
        );
    }

    #[test]
    fn test_unsupported_platform_display() {
        let err = LoadExtensionError::UnsupportedPlatform;
        assert_eq!(
            err.to_string(),
            "Extension loading is not supported on this platform (WASM)"
        );
    }

    #[test]
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
