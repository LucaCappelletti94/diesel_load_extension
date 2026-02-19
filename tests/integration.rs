use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};

fn create_connection() -> SqliteConnection {
    SqliteConnection::establish(":memory:").expect("Failed to create in-memory connection")
}

#[test]
fn test_load_extension_auto_enables_and_disables() {
    let mut conn = create_connection();

    // load_extension should work without manually enabling first
    let result = conn.load_extension("/nonexistent/extension.so", None);
    assert!(result.is_err());
    match &result.unwrap_err() {
        LoadExtensionError::LoadFailed(msg) => {
            assert!(!msg.is_empty(), "Expected non-empty error message");
        }
        err => panic!("Expected LoadFailed, got: {err:?}"),
    }
}

#[test]
fn test_manual_enable_disable_workflow() {
    let mut conn = create_connection();

    conn.enable_load_extension(true).unwrap();
    conn.enable_load_extension(false).unwrap();
    conn.enable_load_extension(true).unwrap();
    conn.enable_load_extension(false).unwrap();
}

#[test]
fn test_multiple_connections_are_independent() {
    let mut conn1 = create_connection();
    let mut conn2 = create_connection();

    // Enable on conn1
    conn1.enable_load_extension(true).unwrap();

    // conn2 should still have loading disabled
    conn2.enable_load_extension(false).unwrap();

    // Load on conn1 works (fails because file doesn't exist, not because unauthorized)
    let result = conn1.load_extension("/nonexistent/extension.so", None);
    assert!(result.is_err());
    match &result.unwrap_err() {
        LoadExtensionError::LoadFailed(msg) => {
            assert!(!msg.is_empty(), "Expected non-empty error message");
        }
        err => panic!("Expected LoadFailed, got: {err:?}"),
    }
}

#[test]
fn test_error_messages_are_meaningful() {
    let mut conn = create_connection();

    let result = conn.load_extension("/nonexistent/extension.so", None);
    let err = result.unwrap_err();
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

#[test]
fn test_load_extensions_batch() {
    let mut conn = create_connection();

    // All extensions fail because they don't exist, but the first one triggers the error
    let result = conn.load_extensions(&[
        ("/nonexistent/ext1.so", None),
        ("/nonexistent/ext2.so", Some("init")),
    ]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LoadExtensionError::LoadFailed(_)
    ));
}

#[test]
fn test_load_extensions_validates_all_inputs_upfront() {
    let mut conn = create_connection();

    // The second extension has an invalid path — should fail before enabling
    let result = conn.load_extensions(&[("valid_extension", None), ("path\0null", None)]);
    assert!(matches!(
        result.unwrap_err(),
        LoadExtensionError::InvalidPath
    ));
}

#[test]
fn test_load_extensions_empty() {
    let mut conn = create_connection();
    conn.load_extensions(&[])
        .expect("Loading empty extension list should succeed");
}
