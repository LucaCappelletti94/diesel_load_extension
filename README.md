# `diesel_load_extension`

[![CI](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml/badge.svg)](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension)

Diesel extension for `SQLite` [`load_extension`](https://www.sqlite.org/c3ref/load_extension.html) support.

This crate provides a safe Rust wrapper around `SQLite`'s [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) for [Diesel](https://diesel.rs)'s `SqliteConnection`. Extension loading is automatically enabled before the load and disabled afterward, so it is never left enabled unintentionally.

## Platform Support

This crate is continuously validated in CI across desktop, mobile, and edge targets.

CI now validates both linkage modes:

- `bundled SQLite`: `sqlite-bundled` feature enabled.
- `system SQLite`: `--no-default-features` (links to platform-provided `sqlite3`).

| Target | Bundled `SQLite` lane | System `SQLite` lane | Guarantee level |
| --- | --- | --- | --- |
| `ubuntu-latest` | `cargo test` | `cargo test --no-default-features` | Runtime |
| `macos-latest` | `cargo test` | `cargo check --tests --no-default-features` | Bundled: Runtime; System: Build-check |
| `windows-latest` | `cargo test` | `cargo test --no-default-features` | Runtime |
| `ubuntu-24.04-arm` (`aarch64-unknown-linux-gnu`) | N/A | `cargo test --no-default-features` | Runtime |
| `aarch64-apple-ios` | `cargo check` | `cargo check --no-default-features` | Build-check |
| `aarch64-apple-ios-sim` | `cargo test` (simulator runner) | `cargo test --no-default-features` (simulator runner) | Runtime |
| `aarch64-linux-android` | `cargo check` + `cargo test --no-run` | `cargo check --no-default-features` + `cargo check --tests --no-default-features` | Bundled: Link/no-run; System: Build-check |
| `armv7-unknown-linux-gnueabihf` | N/A | `cargo check --no-default-features` | Build-check |
| `aarch64-unknown-linux-musl` | N/A | `cargo check --no-default-features` | Build-check |
| `x86_64-unknown-linux-musl` | N/A | `cargo check --no-default-features` | Build-check |
| `aarch64-pc-windows-msvc` | N/A | `cargo test --no-run --target aarch64-pc-windows-msvc --no-default-features` | Link/no-run |

`Runtime` gives end-to-end ABI confidence (including dynamic load behavior in tests).
`Link/no-run` validates cross-target symbol/link compatibility but does not execute on target.
`Build-check` validates compilation and target configuration only.

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
use diesel_load_extension::{SqliteLoadExtensionExt, LoadExtensionError};
# use std::path::Path;
# use std::process::Command;
# use std::time::{SystemTime, UNIX_EPOCH};
# fn extension_binary_name(stem: &str) -> String {
#     #[cfg(target_os = "macos")]
#     {
#         format!("lib{stem}.dylib")
#     }
#     #[cfg(target_os = "windows")]
#     {
#         format!("{stem}.dll")
#     }
#     #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
#     {
#         format!("lib{stem}.so")
#     }
# }
# fn build_smoke_extension(stem: &str) -> String {
#     let source = Path::new(env!("CARGO_MANIFEST_DIR"))
#         .join("tests")
#         .join("fixtures")
#         .join("smoke_extension.c");
#     assert!(source.exists(), "Missing fixture source: {source:?}");
#     let stamp = SystemTime::now()
#         .duration_since(UNIX_EPOCH)
#         .expect("System clock should be after UNIX epoch")
#         .as_nanos();
#     let build_dir = std::env::temp_dir()
#         .join(format!("diesel_load_extension_readme_{stem}_{stamp}"));
#     std::fs::create_dir_all(&build_dir).expect("Failed to create build dir");
#     let extension = build_dir.join(extension_binary_name(stem));
#     let mut cc = Command::new("cc");
#     #[cfg(target_os = "macos")]
#     {
#         cc.arg("-dynamiclib");
#     }
#     #[cfg(target_os = "windows")]
#     {
#         cc.arg("-shared");
#     }
#     #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
#     {
#         cc.args(["-shared", "-fPIC"]);
#     }
#     let status = cc
#         .arg(&source)
#         .arg("-o")
#         .arg(&extension)
#         .status()
#         .expect("Failed to run C compiler");
#     assert!(status.success(), "Failed to compile fixture extension");
#     extension
#         .to_str()
#         .expect("Temporary extension path must be valid UTF-8")
#         .to_owned()
# }
# let extension_path = build_smoke_extension("readme_smoke");
let mut conn = SqliteConnection::establish(":memory:").unwrap();

// Working case with SQLite default entry point lookup (`sqlite3_extension_init`).
conn.load_extension(&extension_path, None).unwrap();

// Working case with explicit entry point.
conn.load_extension(&extension_path, Some("sqlite3_smokeext_init"))
    .unwrap();

// Failure case: missing library path.
let result = conn.load_extension("/nonexistent/extension.so", None);
assert!(matches!(result, Err(LoadExtensionError::LoadFailed { .. })));
```

Call `load_extension` once per extension you need to load.

### When to use `load_extension` vs `sqlite3_auto_extension`

- Use `load_extension` when you want to load an extension library by path for a specific connection, at a specific point in your app's lifecycle.
- Use `sqlite3_auto_extension` when the extension is already linked into your process and you want it auto-registered for every new `SQLite` connection.
- Prefer `load_extension` for explicit, connection-scoped loading in application code.
- Prefer `sqlite3_auto_extension` for embedded/builtin extensions and framework-level global initialization.

### `SQLite` build requirements

This crate calls [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) and [`sqlite3_enable_load_extension`](https://www.sqlite.org/c3ref/enable_load_extension.html), which are **not part of every `SQLite` build**. `SQLite` builds compiled with [`SQLITE_OMIT_LOAD_EXTENSION`](https://www.sqlite.org/compile.html#omit_load_extension) omit these functions entirely, and linking against such a build will fail. With the default `sqlite-bundled` feature, this crate compiles `SQLite` from source with extension loading enabled. If you disable default features to use a system-provided `SQLite` library, ensure it was built without `SQLITE_OMIT_LOAD_EXTENSION`.

## License

MIT — see LICENSE for details.
