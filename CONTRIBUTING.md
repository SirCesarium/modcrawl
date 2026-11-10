# Contributing to modcrawl

First of all, thank you so much for considering contributing to modcrawl! Projects like this rely on the time and effort of community members like you. Whether you are fixing a bug, improving documentation, or proposing a new feature, your help is truly appreciated.

## Language

Code, comments, documentation, and commit messages are written in English. Issues, pull request descriptions, and discussions can be in English or Spanish. The maintainer speaks both.

## Before you start

Open an issue before starting work on:

- a new feature;
- a change to the public Rust API;
- a change to a JSON output contract;
- a new dependency.

Small bug fixes can go straight to a pull request. If the fix is larger than it first looks, open an issue so we can agree on the approach.

Keep each pull request focused on one change. Do not mix a refactor, a feature, and a formatting pass in the same PR.

## Development setup

The repository includes a devcontainer. You can also work locally with a current stable Rust toolchain.

```sh
cargo build
cargo test --all-features
```

The web app lives in `web/` and uses Bun.

```sh
cd web
bun install --frozen-lockfile
```

## Quality gates

Every pull request must pass the same checks that CI runs.

Rust:

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo check --target wasm32-unknown-unknown --no-default-features --features classfile,wasm
```

The WASM check needs the target installed:

```sh
rustup target add wasm32-unknown-unknown
```

Web:

```sh
cd web
bun install --frozen-lockfile
bun run lint
bun run check
bun run test
bun run build
```

`bun run lint` runs Prettier and ESLint. `bun run check` runs `svelte-check`. The web checks run in their own workflow.

## Code rules

- Follow the lints in `Cargo.toml`. They deny `unwrap` and `expect` outside tests.
- Return errors instead of panicking. Use `?` and the error types in `src/error.rs`.
- ‼️ **Library code does not print to stdout!** Rendering belongs in `src/ui`.
- Do not add `unsafe` code without a reason and a maintainer's approval.
- Add tests for behavior changes. Tests must be deterministic and must not need network access.
- Add doc comments to new public APIs and update user-facing documentation when behavior changes.
- Keep the diff focused. Do not reformat code you did not change (unless the CI is failing).
- Do not bump the crate version or edit `Cargo.lock` unless the dependency set changes.

## Dependencies

New dependencies need a reason. Open an issue first and explain:

- what the dependency does;
- why the standard library or an existing dependency is not enough;
- the effect on MSRV, binary size, and feature flags.

Development dependencies follow the same rule, with a little more room for test tooling. Do not update dependencies in an unrelated pull request. Dependabot handles routine updates.

## Workflows and CI

Only maintainers change files under `.github/workflows/`. If an external pull request edits one, it will be closed and redirected to an issue. See [Requesting a CI change](#requesting-a-ci-change) for details.

Dependabot pull requests are the exception. A maintainer reviews and merges them.

## Requesting a CI change

If you need a new build, **another operating system** or **architecture**, **a new installer**, or an **extra check**, **open an issue** instead of a pull request. Use the label that fits:

- `bug` for a broken build or check;
- `security` for a security problem in CI;
- `enhancement` for a new target, installer, or check. Use this label for ideas too.

Explain what you need, why you need it, and which platform or tool it involves. A maintainer decides whether to implement it and opens the workflow change.

## AI-assisted contributions

**You can use AI tools.** The person who opens the pull request is responsible for the code, including the tests and the edge cases.

Add `[🤖]` at the end of the pull request title, for example `feat(cli): add --json to the type command [🤖]`. The marker belongs only in the pull request title, never in commit messages. Whoever opens the pull request, a person or a bot running `gh pr create`, decides whether to add it.

A maintainer will add the `ai-assisted` label. The label helps reviewers look for the mistakes that generated code tends to make, such as ignored formatting rules or thinking-out-loud comments like `// Wait, this won't work`.

### Saving tokens with TOON

[TOON](https://toonformat.dev/) is a compact encoding of JSON for model input. It pays off on uniform data, which is most of what an AI tool reads: issue lists, metadata dumps, command output.

```sh
npx @toon-format/cli file.json
gh issue list --json number,title,body | npx @toon-format/cli
```

`npm install -g @toon-format/cli` adds the shorter `toon` command. The encoding is lossless, so the same tool decodes it back to JSON when you need that.

### MCP servers

Ready-to-copy snippets live in [`.agents/mcps`](./.agents/mcps), one directory per server. Nothing loads them automatically; add a server to your own configuration when you need it:

- OpenCode: `opencode.jsonc`, under `mcp.servers`. `opencode mcp add <name> --url <remote-url>` writes the entry.
- Claude Code: `.mcp.json` at the project root, under `mcpServers`. `claude mcp add --transport http <name> --scope project <url>` writes it.
- Antigravity CLI: `.agents/mcp_config.json` for the workspace, or `~/.gemini/config/mcp_config.json` globally, under `mcpServers`.

[`.agents/mcps/README.md`](./.agents/mcps/README.md) has the exact shape for each client. Keep credentials out of committed files: environment variables where the client expands them, and the global config where it does not.

Fully automated bots must contact the maintainer before opening pull requests. Do not send bulk or unreviewed PRs. Do not paste secrets, private code, or third-party code without a compatible license.

## Commit conventions

Commits are written in English and follow [Conventional Commits](https://www.conventionalcommits.org/). Use a type, an optional scope, and a short subject:

```text
feat(cli): add --json to the type command
fix(loaders): report NeoForge 1.20 mods correctly
docs: explain the AI disclosure rule
test: add Fabric dependency fixtures
```

Common types are `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `chore`, `revert`, and `ci`. **Mark a breaking change with** `!` and a `BREAKING CHANGE:` footer:

```text
feat(metadata)!: flatten the JSON schema

BREAKING CHANGE: metadata --json now emits a flat object.
```

Pull request titles should be in English.

## Reporting bugs, features, and security issues

Use the issue templates for bugs and feature requests. Pick the label that fits:

- `bug` for bugs;
- `security` for security reports;
- `enhancement` for features and ideas;
- `documentation` for documentation changes;
- `question` for usage questions.

Maintainers apply the remaining labels:

- `good first issue` and `help wanted` mark tasks for contributors;
- `dependencies`, `rust`, and `github_actions` mark the area;
- `duplicate`, `invalid`, and `wontfix` close issues that will not proceed.

## Review

A maintainer may ask for changes or close a pull request that does not fit the project. You will get a reason. Small pull requests with tests and a clear description are easier to review and merge.

## License

By contributing, you agree that your contribution is licensed under the [MIT license](./LICENSE).
