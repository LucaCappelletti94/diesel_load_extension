use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::path::{Path, PathBuf};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::process::Command;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::time::{SystemTime, UNIX_EPOCH};

fn create_connection() -> SqliteConnection {
    SqliteConnection::establish(":memory:").expect("Failed to create in-memory connection")
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn test_extension_source_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("smoke_extension.c")
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn extension_binary_name(stem: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("lib{stem}.dylib")
    }
    #[cfg(target_os = "windows")]
    {
        format!("{stem}.dll")
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        format!("lib{stem}.so")
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn compile_test_extension(stem: &str) -> PathBuf {
    let source = test_extension_source_path();
    assert!(
        source.exists(),
        "Missing test extension source: {}",
        source.display()
    );

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock should be after UNIX epoch")
        .as_nanos();
    let out_dir = std::env::temp_dir().join(format!("diesel_load_extension_{stem}_{stamp}"));
    std::fs::create_dir_all(&out_dir).expect("Failed to create temporary build directory");

    let output = out_dir.join(extension_binary_name(stem));

    let mut cmd = Command::new("cc");
    #[cfg(target_os = "macos")]
    {
        cmd.arg("-dynamiclib");
    }
    #[cfg(target_os = "windows")]
    {
        cmd.arg("-shared");
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        cmd.args(["-shared", "-fPIC"]);
    }

    let status = cmd
        .arg(&source)
        .arg("-o")
        .arg(&output)
        .status()
        .expect("Failed to spawn C compiler");
    assert!(
        status.success(),
        "C compiler failed building {}",
        source.display()
    );

    output
}

#[test]
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn test_load_extension_success_with_default_entrypoint() {
    let extension_path = compile_test_extension("smoke_default");
    let extension_path = extension_path
        .to_str()
        .expect("Temporary extension path must be valid UTF-8");

    let mut conn = create_connection();
    conn.load_extension(extension_path, None)
        .expect("Expected successful extension load with default entry point");
}

#[test]
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn test_load_extension_success_with_explicit_entrypoint() {
    let extension_path = compile_test_extension("smoke_explicit");
    let extension_path = extension_path
        .to_str()
        .expect("Temporary extension path must be valid UTF-8");

    let mut conn = create_connection();
    conn.load_extension(extension_path, Some("sqlite3_smokeext_init"))
        .expect("Expected successful extension load with explicit entry point");
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
