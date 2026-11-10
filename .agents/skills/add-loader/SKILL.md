---
name: add-loader
description: Add support for a new Minecraft mod loader or server plugin platform. Use when asked to detect, parse metadata for, or extract dependencies from a loader modcrawl does not support yet.
---

# Adding a loader

Work through `src/core`. A loader does not touch `src/ui` or `src/cmd`.

## 1. Type

Add a variant to `ModType`, `ForgeModFormat`, or `PluginType` in
`src/core/detect_mod_type.rs`, plus its `Display` arm. The display string is
user-facing and reaches JSON output.

## 2. Module

Create `src/core/loaders/<loader>/`:

```text
mod.rs        ModHandler implementation
metadata.rs   serde types and parse()
dep.rs        dependency extraction
```

`fabric/` is the smallest complete example. `neoforge/` shows the special cases
for `mods.toml`.

## 3. Handler

Implement `ModHandler` in `mod.rs`:

- `detection_files` returns the entry names that identify the loader.
- `metadata_files` overrides it only when the file read differs from the file
  detected.
- `read_metadata` parses the raw file into `ModMetadata`.
- `extract_deps` returns `Vec<DepEntry>`, or keeps the default when the loader
  has no dependency metadata.

## 4. Registration

Add the module to `src/core/loaders/mod.rs` and register the handler in
`register_all`. Order is priority: the first handler with a matching detection
file wins. The current order is Paper, Bukkit, NeoForge, Forge (`mods.toml`),
Forge legacy (`mcmod.info`), Fabric, Quilt, Velocity, Bungee. Place the new
handler where its detection file could collide with an existing one.

## 5. Tests

- Extend the `test_handler` table in `src/core/loaders/mod.rs` so the
  registration order stays honest.
- Unit-test `metadata::parse` and `dep::extract` with inline metadata strings.
- Add an integration test in `tests/` that builds a JAR with `TempJar` from
  `tests/common` and asserts the detected type, the metadata, and the
  dependencies.

No network. No committed JARs.

## 6. Docs

Add the loader to the supported table in `README.md`.

## 7. Verify

```sh
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
```
