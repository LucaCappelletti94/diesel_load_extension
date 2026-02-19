#![cfg(target_arch = "wasm32")]

use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn create_connection() -> SqliteConnection {
    SqliteConnection::establish(":memory:").expect("Failed to create in-memory connection")
}

#[wasm_bindgen_test]
fn test_register_auto_extension_idempotent() {
    use diesel_load_extension::wasm::register_auto_extension;

    unsafe extern "C" fn dummy_init(
        _db: *mut sqlite_wasm_rs::sqlite3,
        _pz_err_msg: *mut *mut std::ffi::c_char,
        _p_api: *const sqlite_wasm_rs::sqlite3_api_routines,
    ) -> std::ffi::c_int {
        0
    }

    register_auto_extension(dummy_init);
    register_auto_extension(dummy_init);

    let _ = create_connection();
}

#[wasm_bindgen_test]
fn test_enable_load_extension_fails_on_wasm() {
    let mut conn = create_connection();
    let result = conn.enable_load_extension(true);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), LoadExtensionError::UnsupportedPlatform),
        "Expected UnsupportedPlatform on WASM"
    );
}

#[wasm_bindgen_test]
fn test_disable_load_extension_fails_on_wasm() {
    let mut conn = create_connection();
    let result = conn.enable_load_extension(false);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), LoadExtensionError::UnsupportedPlatform),
        "Expected UnsupportedPlatform on WASM"
    );
}

#[wasm_bindgen_test]
fn test_load_extension_fails_on_wasm() {
    let mut conn = create_connection();
    let result = conn.load_extension("some_extension", None);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), LoadExtensionError::UnsupportedPlatform),
        "Expected UnsupportedPlatform on WASM"
    );
}

#[wasm_bindgen_test]
fn test_load_extension_with_entry_point_fails_on_wasm() {
    let mut conn = create_connection();
    let result = conn.load_extension("some_extension", Some("my_init"));
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), LoadExtensionError::UnsupportedPlatform),
        "Expected UnsupportedPlatform on WASM"
    );
}

#[wasm_bindgen_test]
fn test_invalid_path_null_byte_on_wasm() {
    let mut conn = create_connection();
    // Null byte validation happens before the UnsupportedPlatform check.
    let result = conn.load_extension("path\0null", None);
    assert!(matches!(
        result.unwrap_err(),
        LoadExtensionError::InvalidPath
    ));
}

#[wasm_bindgen_test]
fn test_invalid_entry_point_null_byte_on_wasm() {
    let mut conn = create_connection();
    // Null byte validation happens before the UnsupportedPlatform check.
    let result = conn.load_extension("some_extension", Some("entry\0null"));
    assert!(matches!(
        result.unwrap_err(),
        LoadExtensionError::InvalidEntryPoint
    ));
}

#[wasm_bindgen_test]
fn test_load_extensions_fails_on_wasm() {
    let mut conn = create_connection();
    let result = conn.load_extensions(&[("some_extension", None)]);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), LoadExtensionError::UnsupportedPlatform),
        "Expected UnsupportedPlatform on WASM"
    );
}

#[wasm_bindgen_test]
fn test_load_extensions_empty_on_wasm() {
    let mut conn = create_connection();
    // Empty list returns Ok(()) without attempting to enable extension loading.
    conn.load_extensions(&[])
        .expect("Loading empty extension list should succeed on WASM");
}
