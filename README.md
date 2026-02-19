# `diesel_load_extension`

[![CI](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml/badge.svg)](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension)

Diesel extension for `SQLite` [`load_extension`](https://www.sqlite.org/c3ref/load_extension.html) support.

This crate provides a safe Rust wrapper around `SQLite`'s [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) for [Diesel](https://diesel.rs)'s `SqliteConnection`. Extension loading is automatically enabled before the load and disabled afterward, so it is never left enabled unintentionally.

Note: not all `SQLite` builds include the load-extension ABI. Builds compiled with `SQLITE_OMIT_LOAD_EXTENSION` omit these symbols entirely, and linking will fail if they are missing.

Diesel does not currently expose access to the raw `SQLite` connection handle (`*mut sqlite3`). This crate depends on a proposed [`with_raw_connection`](https://github.com/diesel-rs/diesel/pull/4966) API that provides scoped access to the underlying C handle, enabling features like extension loading without requiring Diesel to wrap every optional `SQLite` C API.

Until [diesel-rs/diesel#4966](https://github.com/diesel-rs/diesel/pull/4966) is merged, this crate depends on a fork of Diesel and is **not intended for crates.io publication yet**. Once that PR lands, this crate will switch to a released Diesel version and be published normally.

## Usage

Since this crate is not yet on crates.io, add both it and the Diesel fork as git dependencies:

```toml
[dependencies]
diesel_load_extension = { git = "https://github.com/LucaCappelletti94/diesel_load_extension" }
diesel = { git = "https://github.com/LucaCappelletti94/diesel", branch = "sqlite-session-changeset", features = ["sqlite"] }
```

Then use the extension trait:

```rust
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};

fn main() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();

    // Load an extension (e.g., SpatiaLite)
    // Extension loading is automatically enabled and disabled around the call.
    let result = conn.load_extension("mod_spatialite", None);
    assert!(matches!(result, Err(LoadExtensionError::LoadFailed(_))));
}
```

### Loading Multiple Extensions

When loading several extensions, use `load_extensions` to enable and disable extension loading only once:

```rust
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};

let mut conn = SqliteConnection::establish(":memory:").unwrap();
let result = conn.load_extensions(&[
    ("mod_spatialite", None),
    ("my_extension", Some("my_extension_init")),
]);
assert!(matches!(result, Err(LoadExtensionError::LoadFailed(_))));
```

### Custom Entry Points

If your extension uses a non-default entry point function, you can specify it:

```rust
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};

let mut conn = SqliteConnection::establish(":memory:").unwrap();
let result = conn.load_extension("my_extension", Some("my_extension_init"));
assert!(matches!(result, Err(LoadExtensionError::LoadFailed(_))));
```

### Error Handling

The crate provides a [`LoadExtensionError`] type that surfaces `SQLite` error messages:

```rust
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{SqliteLoadExtensionExt, LoadExtensionError};

let mut conn = SqliteConnection::establish(":memory:").unwrap();

let err = conn.load_extension("nonexistent_extension", None).unwrap_err();
assert!(matches!(err, LoadExtensionError::LoadFailed(_)));
```

### WASM Extensions (Precompiled)

On `wasm32-unknown-unknown`, dynamic library loading is unavailable. You can still use
`SQLite` extensions that are **compiled into** your WASM build by registering them with
`sqlite3_auto_extension`. This crate exposes a safe helper for that. This does **not**
load external `.so/.dll/.dylib` files; it only registers precompiled extensions.

```rust
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
{
    use diesel::prelude::*;
    use diesel::SqliteConnection;
    use diesel_load_extension::wasm::register_auto_extension;

    unsafe extern "C" fn geolite_init(
        _db: *mut sqlite_wasm_rs::sqlite3,
        _pz_err_msg: *mut *mut std::ffi::c_char,
        _p_api: *const sqlite_wasm_rs::sqlite3_api_routines,
    ) -> std::ffi::c_int {
        // Call into your extension's initialization routine here.
        0
    }

    register_auto_extension(geolite_init);

    // This works with the default in-memory database on WASM.
    // If you need OPFS or another VFS, configure it using sqlite-wasm-rs APIs.
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    let _ = &mut conn;
}
```

### `SQLite` build requirements

This crate calls [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) and [`sqlite3_enable_load_extension`](https://www.sqlite.org/c3ref/enable_load_extension.html), which are **not part of every `SQLite` build**. `SQLite` builds compiled with [`SQLITE_OMIT_LOAD_EXTENSION`](https://www.sqlite.org/compile.html#omit_load_extension) omit these functions entirely, and linking against such a build will fail. This crate depends on `libsqlite3-sys` with the `bundled` feature, which compiles `SQLite` from source with extension loading enabled, so this is not an issue with the default configuration. If you switch to a system-provided `SQLite` library, ensure it was built without `SQLITE_OMIT_LOAD_EXTENSION`.

## License

MIT — see LICENSE for details.
