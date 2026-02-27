# `diesel_load_extension`

[![CI](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml/badge.svg)](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension)

Diesel extension for `SQLite` [`load_extension`](https://www.sqlite.org/c3ref/load_extension.html) support.

This crate provides a safe Rust wrapper around `SQLite`'s [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) for [Diesel](https://diesel.rs)'s `SqliteConnection`. Extension loading is automatically enabled before the load and disabled afterward, so it is never left enabled unintentionally.

Note: not all `SQLite` builds include the load-extension ABI. Builds compiled with `SQLITE_OMIT_LOAD_EXTENSION` omit these symbols entirely, and linking will fail if they are missing.

This crate is native-only. On `wasm32-unknown-unknown`, use
[`sqlite-wasm-rs`](https://crates.io/crates/sqlite-wasm-rs) directly with
`sqlite3_auto_extension` for precompiled extensions.

This crate depends on Diesel's `with_raw_connection` API to access the raw `SQLite`
connection handle (`*mut sqlite3`) in a scoped and safe way.

`with_raw_connection` is available on Diesel's `main` branch and may not yet be
available in the latest crates.io release.

Because of that dependency, this crate is currently configured as `publish = false`.

## Usage

Add dependencies as follows:

```toml
[dependencies]
diesel_load_extension = { git = "https://github.com/LucaCappelletti94/diesel_load_extension" }
diesel = { git = "https://github.com/diesel-rs/diesel", branch = "main", features = ["sqlite"] }
```

### `SQLite` Linkage Modes

By default, this crate enables the `sqlite-bundled` feature, which compiles
`SQLite` from source via `libsqlite3-sys`.

To use a system-provided `SQLite` library instead:

```toml
[dependencies]
diesel_load_extension = { git = "https://github.com/LucaCappelletti94/diesel_load_extension", default-features = false }
diesel = { git = "https://github.com/diesel-rs/diesel", branch = "main", features = ["sqlite"] }
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
    assert!(matches!(result, Err(LoadExtensionError::LoadFailed { .. })));
}
```

Call `load_extension` once per extension you need to load.

### Custom Entry Points

If your extension uses a non-default entry point function, you can specify it:

```rust
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{LoadExtensionError, SqliteLoadExtensionExt};

let mut conn = SqliteConnection::establish(":memory:").unwrap();
let result = conn.load_extension("my_extension", Some("my_extension_init"));
assert!(matches!(result, Err(LoadExtensionError::LoadFailed { .. })));
```

### Error Handling

The crate provides a [`LoadExtensionError`] type that surfaces `SQLite` error messages:

```rust
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_load_extension::{SqliteLoadExtensionExt, LoadExtensionError};

let mut conn = SqliteConnection::establish(":memory:").unwrap();

let err = conn.load_extension("nonexistent_extension", None).unwrap_err();
assert!(matches!(err, LoadExtensionError::LoadFailed { .. }));
```

### `SQLite` build requirements

This crate calls [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) and [`sqlite3_enable_load_extension`](https://www.sqlite.org/c3ref/enable_load_extension.html), which are **not part of every `SQLite` build**. `SQLite` builds compiled with [`SQLITE_OMIT_LOAD_EXTENSION`](https://www.sqlite.org/compile.html#omit_load_extension) omit these functions entirely, and linking against such a build will fail. With the default `sqlite-bundled` feature, this crate compiles `SQLite` from source with extension loading enabled. If you disable default features to use a system-provided `SQLite` library, ensure it was built without `SQLITE_OMIT_LOAD_EXTENSION`.

## License

MIT — see LICENSE for details.
