# diesel_load_extension

[![CI](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml/badge.svg)](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension)

Diesel extension for SQLite [`load_extension`](https://www.sqlite.org/c3ref/load_extension.html) support.

This crate provides a safe Rust wrapper around SQLite's [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) for [Diesel](https://diesel.rs)'s `SqliteConnection`. Extension loading is automatically enabled before the load and disabled afterward, so it is never left enabled unintentionally.

## Why a separate crate?

Diesel does not currently expose access to the raw SQLite connection handle (`*mut sqlite3`). This crate depends on a proposed [`with_raw_connection`](https://github.com/diesel-rs/diesel/pull/4966) API that provides scoped access to the underlying C handle, enabling features like extension loading without requiring Diesel to wrap every optional SQLite C API.

Until [diesel-rs/diesel#4966](https://github.com/diesel-rs/diesel/pull/4966) is merged, this crate depends on a fork of Diesel and **cannot be published to crates.io**. Once that PR lands, this crate will switch to a released Diesel version and be published normally.

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

The crate provides a [`LoadExtensionError`] type that surfaces SQLite error messages:

```rust
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{SqliteLoadExtensionExt, LoadExtensionError};

let mut conn = SqliteConnection::establish(":memory:").unwrap();

let err = conn.load_extension("nonexistent_extension", None).unwrap_err();
assert!(matches!(err, LoadExtensionError::LoadFailed(_)));
```

## API

### `SqliteLoadExtensionExt` trait

| Method                                | Description                                                                                                                              |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `load_extension(path, entry_point)`   | Load an extension from a shared library. Automatically enables and disables extension loading. Wraps [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html). |
| `load_extensions(extensions)`         | Load multiple extensions in a single enable/disable cycle.                                                                               |
| `enable_load_extension(enabled: bool)`| Manually enable or disable extension loading. Wraps [`sqlite3_enable_load_extension`](https://www.sqlite.org/c3ref/enable_load_extension.html). |

### `LoadExtensionError` enum

| Variant                | Description                                                                                                  |
| --------------------- | ------------------------------------------------------------------------------------------------------------ |
| `EnableFailed(String)` | `sqlite3_enable_load_extension` returned a non-OK status, with the SQLite error message.                     |
| `LoadFailed(String)`   | `sqlite3_load_extension` failed, with the SQLite error message.                                              |
| `InvalidPath`          | The extension path contains an interior null byte.                                                           |
| `InvalidEntryPoint`    | The entry point name contains an interior null byte.                                                         |
| `UnsupportedPlatform`  | Extension loading is not available on this platform (WASM).                                                   |

## Platform Support

| Platform                         | Status                                   |
| ------------------------------- | ---------------------------------------- |
| Linux                            | Full support                             |
| macOS                            | Full support                             |
| Windows                          | Full support                             |
| WASM (`wasm32-unknown-unknown`)  | Compiles, but extension loading is unavailable |

On native targets, extension loading works via `dlopen`/`LoadLibrary`. On WASM targets, extension loading is not supported since dynamic library loading is unavailable in WebAssembly. All methods return `LoadExtensionError::UnsupportedPlatform` on WASM (except `load_extensions` with an empty list, which returns `Ok(())`). Input validation (null bytes) is still performed before the platform check, so callers get specific errors for invalid inputs regardless of platform.

### SQLite build requirements

This crate calls [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) and [`sqlite3_enable_load_extension`](https://www.sqlite.org/c3ref/enable_load_extension.html), which are **not part of every SQLite build**. SQLite builds compiled with [`SQLITE_OMIT_LOAD_EXTENSION`](https://www.sqlite.org/compile.html#omit_load_extension) omit these functions entirely, and linking against such a build will fail. This crate depends on `libsqlite3-sys` with the `bundled` feature, which compiles SQLite from source with extension loading enabled, so this is not an issue with the default configuration. If you switch to a system-provided SQLite library, ensure it was built without `SQLITE_OMIT_LOAD_EXTENSION`.

## Development

### Pre-commit hook

This repository includes a pre-commit hook that runs `cargo fmt --check` and `cargo clippy`. To enable it:

```bash
git config core.hooksPath .githooks
```

## License

MIT — see [LICENSE](LICENSE) for details.
