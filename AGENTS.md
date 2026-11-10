# modcrawl

Rust library, CLI, and WASM/npm package that inspects Minecraft mod and plugin
JARs: loader detection, metadata, dependencies, and class-file analysis.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before contributing. It is the source of
truth for pull requests, dependency requests, licensing, and review.

## Layout

| Path | Contents |
| --- | --- |
| `src/core` | Analysis: `loaders/`, `metadata/`, `dep/`, `classfile/`, `side`, `identify`, `registry`. |
| `src/cmd` | Clap subcommands, one module per command. Feature `cli`. |
| `src/ui` | Rendering and every write to the terminal. |
| `src/wasm.rs` | `wasm-bindgen` exports. |
| `tests/` | Integration tests; `TempJar` in `tests/common`. |
| `fuzz/` | `cargo-fuzz` targets. |
| `web/` | SvelteKit app. See `web/AGENTS.md`. |
| `scripts/` | `fetch_modrinth_fixtures.sh`, which fills `.smoke_fixtures/`. |

Rust 1.99.1, edition 2024. The toolchain is pinned by `rust-version` in
`Cargo.toml`.

## Commands

These mirror CI exactly. Run them from the repository root.

```sh
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo check --all-features --locked
cargo check --target wasm32-unknown-unknown --no-default-features --features classfile,wasm
cargo audit
cargo deny check
```

Web, from `web/`:

```sh
bun install --frozen-lockfile
bun run lint
bun run check
bun run test
bun run build
```

Run the full set when you finish a task or when the user asks for a pull
request. Do not run it after every prompt.

## CLI commands

| Subcommand | Module | WASM export |
| --- | --- | --- |
| `dep` | `src/cmd/dep.rs` | `deps_json` |
| `metadata` | `src/cmd/metadata.rs` | `metadata_json` |
| `type` | `src/cmd/type.rs` | `identify` |
| `side` | `src/cmd/side.rs` | `side_json` |
| `classes` | `src/cmd/classes.rs` | `classes_json` |
| `grep` | `src/cmd/grep.rs` | `grep_json` |
| `mixins` | `src/cmd/mixins.rs` | `mixins_json` |
| `dupes` | `src/cmd/dupes.rs` | `dupes_json` |

## Hard rules

- **The library never writes to the terminal.** `println!`, `print!`,
  `eprintln!`, and `dbg!` are forbidden in `src/core`, `src/lib.rs`, and
  `src/error.rs`. `dbg!` is allowed only while debugging a local change and must
  not survive the commit. Output belongs in `src/ui`, which `--json` bypasses.
- **Never edit `.github/workflows`.** Only the maintainer changes them.
- **No new dependency without a prior issue and maintainer approval.** If a task
  seems to need one, ask before touching `Cargo.toml`: "I plan to add X, but
  CONTRIBUTING requires a prior issue. Do you approve? You will need to open the
  issue if it does not exist yet."
- **Every CLI command is exported in `src/wasm.rs`** and listed in the table
  above.
- **Tests are deterministic and never touch the network.** Fixtures come from
  `scripts/fetch_modrinth_fixtures.sh`; JARs are never committed.
- **No `unwrap()` or `expect()` outside tests.** Both are denied crate-wide; test
  modules opt in with `#[cfg(test)]` and
  `#[allow(clippy::unwrap_used, clippy::expect_used)]`.
- **Clippy pedantic is denied, not advisory.** The lints that bite here are
  `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`,
  `module_name_repetitions`, and `needless_pass_by_value`.
- **Doc comments on public items other code depends on, and on every module.**
  Skip helpers whose name and signature already say everything.
- **No thinking-out-loud comments.** A comment exists for a constraint, an
  invariant, a non-obvious decision, or a rejected alternative. If it restates
  the line below it, delete it.

#### Examples of **forbidden** thinking-out-loud comments:

```rust
// Wait, this won't work
// Let me try another approach
// Actually, I need to handle this first
// Hmm, this is tricky
// As an AI, I should check the types
// I'll now add the test
```

## Skills

Task-specific instructions live in `.agents/skills/<id>/SKILL.md`. Read the
matching one before starting:

- `svelte` — writing or reviewing Svelte and SvelteKit code in `web/`.
- `add-loader` — adding support for a mod loader or plugin type.
- `new-command` — adding a CLI subcommand with its UI, `--json` output, and WASM
  export.

## MCPs

Configuration snippets live in `.agents/mcps/<server>/mcp.jsonc`, written for
OpenCode. `.agents/mcps/README.md` gives the same servers for Claude Code
(`.mcp.json`) and Antigravity CLI (`.agents/mcp_config.json` or the global
`~/.gemini/config/mcp_config.json`). Nothing loads them automatically, and no
server is committed as active configuration.

If a task would clearly benefit from an MCP that is not installed, suggest it to
the user. Do not configure it yourself.

## AI-assisted work

Keep tool output small. Context you do not need costs as much as context you do.

- Use `git grep -l <symbol>` when you only need the files that contain it, and
  scope the search to the tree you are changing: `git grep -n <pattern> -- src/`.
  Use `-- web/` or `-- tests/` when the question is about those trees.
- Narrow output at the source: `git log -n 10`, `gh pr list --limit 5`, `rg -m 20`.
- Request only the JSON fields you need instead of every field, as in
  `gh issue list --json number,title`.
- Encode JSON as TOON before reading it into context; it carries the same data in
  fewer tokens for uniform arrays.

```sh
npx @toon-format/cli file.json
gh issue list --json number,title,body | npx @toon-format/cli
```

`npm install -g @toon-format/cli` installs the shorter `toon` command.

Do not add `[🤖]` to commit messages. The marker belongs only in the pull request
title, and whoever opens the PR decides whether it applies.

## Working on tasks

- If the message is a question, investigate and answer without editing files.
- Stay within the scope of the request. Mention adjacent problems instead of
  fixing them.
- Fix the cause, not the symptom.
- Say what you could not verify. Do not claim a fix works when it needed the app
  or a fixture you did not run.
- Do not reformat code you did not change.

## Commits

Conventional Commits with a type, an optional scope, and a short subject. Mark a
breaking change with `!` and a `BREAKING CHANGE:` footer. The full list is in
[CONTRIBUTING.md](CONTRIBUTING.md#commit-conventions).
