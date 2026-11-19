---
title: "Commands"
description: "What each command answers, and the options they share."
topics:
  - "CLI"
  - "Commands"
section: "Getting Started"
---

Every command answers one question about a JAR. All of them accept files,
directories, or standard input, and all of them accept `--json`.

## Commands

| Command | Aliases | Answers |
| --- | --- | --- |
| `metadata` | `meta`, `md` | What does the mod or plugin say about itself? |
| `loader` | `t`, `ty`, `type` | Which loader or plugin platform owns the JAR? |
| `dep` | `d`, `deps`, `dependencies` | Which dependencies are required, embedded, or missing? |
| `side` | `sd` | Is it `client`, `server`, `both`, or `unknown`? |
| `classes` | `c`, `cls` | Which classes does it ship, and which Java version do they target? |
| `grep` | `g`, `search` | Where is this class, method, field, or string referenced? |
| `mixins` | `m`, `mixin` | Which targets do its mixins touch, and which overlap across JARs? |
| `dupes` | `dp`, `duplicate` | Which classes appear in more than one JAR? |

<!-- TODO: link each row to /docs/<slug> once its page exists, and register it in docs/index.json. -->

## Common options

A directory contributes its direct `.jar` children unless you ask for more:

- `-r, --recursive`: walk subdirectories.
- `--depth <N>`: stop after `N` levels; implies `--recursive`, and `1` means
  direct children only.
- `--exclude <PATTERN>`: skip files and folders whose name matches; `*` matches
  any characters, and the flag is repeatable.
- `-j, --json`: print structured JSON instead of text.
- `-`: read from standard input, either a JAR archive or a newline-separated
  list of paths.

## Next

Start with [`metadata`](/docs/metadata). It tells you what a JAR says about
itself, and every other answer builds on that.
