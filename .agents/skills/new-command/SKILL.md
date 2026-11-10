---
name: new-command
description: Add a CLI subcommand to modcrawl with its rendering module, --json output, tests, and WASM export. Use when asked for a new modcrawl command or flag that reads or analyzes JAR data.
---

# Adding a command

Six files change. `src/cmd/metadata.rs` with `src/ui/metadata.rs` is the
reference implementation; read both first.

## 1. Core

Add the analysis in `src/core`, not in `src/cmd`. Expose two entry points: one
taking a path and one taking `&mut impl Read`, named `foo` and `foo_reader`,
mirroring `identify` and `identify_reader`. Return typed data and never print.

## 2. Command

Create `src/cmd/<name>.rs`:

- `Args` derives `clap::Args`, with a `--json` flag when the output is
  structured.
- `run(&Args) -> Result<()>` collects inputs with `collect_sources`, reports
  per-file errors with `ui::file_error`, and returns `Error::Failed` when any
  file failed.

Register the variant in `Commands` in `src/cmd/mod.rs` and dispatch it in
`src/main.rs`. The doc comment on the variant becomes `--help`. Add aliases when
the name is awkward to type. Copy the `#[cfg(feature = "classfile")]` gate from
its siblings when the command reads class files.

## 3. Rendering

Create `src/ui/<name>.rs` and re-export it from `src/ui/mod.rs`. All printing
happens here, through `anstream` and the helpers in `src/ui/table.rs` and
`src/ui/style.rs`. The `--json` path serializes the core types directly and
skips this module.

## 4. WASM

Export `<name>_json` in `src/wasm.rs`: take bytes, call the `_reader` function,
serialize with `serde_json::to_string`, and map errors through the existing
`to_js` helper. A command without an export breaks the invariant in `AGENTS.md`.

## 5. Tests

- Unit-test argument handling and pure helpers in `src/cmd/<name>.rs`.
- Add an integration test in `tests/` using `TempJar`.
- Cover the empty input, invalid JAR, and multiple file paths.

## 6. Docs

Add the command to the quickstart in `README.md` when a user would run it
directly.

## 7. Verify

```sh
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
cargo check --target wasm32-unknown-unknown --no-default-features --features classfile,wasm
```
