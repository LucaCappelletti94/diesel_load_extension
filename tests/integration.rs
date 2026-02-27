use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};

fn create_connection() -> SqliteConnection {
    SqliteConnection::establish(":memory:").expect("Failed to create in-memory connection")
}

#[test]
fn test_load_extension_reports_path_and_message() {
    let mut conn = create_connection();

    let result = conn.load_extension("/nonexistent/extension.so", None);
    assert!(result.is_err());
    match &result.unwrap_err() {
        LoadExtensionError::LoadFailed { path, message } => {
            assert_eq!(path, "/nonexistent/extension.so");
            assert!(!message.is_empty(), "Expected non-empty error message");
        }
        err => panic!("Expected LoadFailed, got: {err:?}"),
    }
}

#[test]
fn test_error_messages_are_meaningful() {
    let mut conn = create_connection();

    let err = conn
        .load_extension("/nonexistent/extension.so", None)
        .unwrap_err();
    let msg = err.to_string();

    assert!(!msg.is_empty(), "Error message should not be empty");
    assert!(
        msg.contains("Failed to load extension"),
        "Error message should mention loading failure, got: {msg}"
    );
}

#[test]
fn test_empty_path() {
    let mut conn = create_connection();
    let result = conn.load_extension("", None);
    assert!(result.is_err());
}

#[test]
fn test_invalid_inputs() {
    let mut conn = create_connection();

    let result = conn.load_extension("path\0null", None);
    assert!(matches!(
        result.unwrap_err(),
        LoadExtensionError::InvalidPath
    ));

    let result = conn.load_extension("valid_path", Some("entry\0null"));
    assert!(matches!(
        result.unwrap_err(),
        LoadExtensionError::InvalidEntryPoint
    ));
}
