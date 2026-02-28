# `diesel_load_extension`

[![CI](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml/badge.svg)](https://github.com/LucaCappelletti94/diesel_load_extension/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94/diesel_load_extension)

Diesel extension for `SQLite` [`load_extension`](https://www.sqlite.org/c3ref/load_extension.html) support.

This crate provides a safe Rust wrapper around `SQLite`'s [`sqlite3_load_extension`](https://www.sqlite.org/c3ref/load_extension.html) for [Diesel](https://diesel.rs)'s `SqliteConnection`. Extension loading is automatically enabled before the load and disabled afterward, so it is never left enabled unintentionally.

## Platform Support

This crate requires two `SQLite` ABI symbols when using system linkage:

- `sqlite3_enable_load_extension`
- `sqlite3_load_extension`

CI validates those symbols as follows:

- `Available`: system-linked lane links and/or runs successfully.
- `Unavailable`: CI environment is missing symbols or missing linkable `libsqlite3`.
- `Not yet validated`: CI currently compiles only; no system ABI link/runtime proof yet.

| Target | System ABI status in CI | Recommended mode | Current CI coverage |
| --- | --- | --- | --- |
| `ubuntu-latest` | Available | Bundled or system | Bundled runtime (`cargo test`) + system runtime (`cargo test --no-default-features`) |
| `macos-latest` | Unavailable (runner `libsqlite3` misses required symbols) | Bundled for runtime | Bundled runtime + system build-check (`cargo check --tests --no-default-features`) |
| `windows-latest` | Available (via `vcpkg` `sqlite3`) | Bundled or system | Bundled runtime + system runtime |
| `ubuntu-24.04-arm` (`aarch64-unknown-linux-gnu`) | Available | System supported in CI | System runtime (`cargo test --no-default-features`) |
| `aarch64-apple-ios` | Not yet validated | Bundled preferred | Bundled/system build-check (`cargo check`) |
| `aarch64-apple-ios-sim` | Unavailable in CI (system symbols missing) | Bundled for runtime | Bundled runtime + system build-check (`cargo check --tests --no-default-features`) |
| `aarch64-linux-android` | Unavailable in CI (NDK lane has no linkable `-lsqlite3`) | Bundled for link checks | Bundled link/no-run (`cargo test --no-run`) + system build-check (`cargo check --tests --no-default-features`) |
| `armv7-unknown-linux-gnueabihf` | Not yet validated | Undecided | System build-check only |
| `aarch64-unknown-linux-musl` | Not yet validated | Undecided | System build-check only |
| `x86_64-unknown-linux-musl` | Not yet validated | Undecided | System build-check only |
| `aarch64-pc-windows-msvc` | Available (via `vcpkg` `sqlite3`) | System supported in CI | System link/no-run (`cargo test --no-run --target ... --no-default-features`) |

Any target not listed in this table is not included in CI yet.

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
# fn c_compiler() -> Option<Command> {
#     for compiler in ["cc", "clang", "gcc"] {
#         if Command::new(compiler).arg("--version").status().is_ok() {
#             return Some(Command::new(compiler));
#         }
#     }
#     None
# }
# fn build_smoke_extension(stem: &str) -> Option<String> {
#     let source = Path::new(env!("CARGO_MANIFEST_DIR"))
#         .join("tests")
#         .join("fixtures")
#         .join("smoke_extension.c");
#     if !source.exists() {
#         return None;
#     }
#     let stamp = SystemTime::now()
#         .duration_since(UNIX_EPOCH)
#         .expect("System clock should be after UNIX epoch")
#         .as_nanos();
#     let build_dir = std::env::temp_dir()
#         .join(format!("diesel_load_extension_readme_{stem}_{stamp}"));
#     std::fs::create_dir_all(&build_dir).expect("Failed to create build dir");
#     let extension = build_dir.join(extension_binary_name(stem));
#     let mut cc = c_compiler()?;
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
#         .status();
#     if !status.map(|s| s.success()).unwrap_or(false) {
#         return None;
#     }
#     Some(
#         extension
#             .to_str()
#             .expect("Temporary extension path must be valid UTF-8")
#             .to_owned(),
#     )
# }
# let Some(extension_path) = build_smoke_extension("readme_smoke") else {
#     return;
# };
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
