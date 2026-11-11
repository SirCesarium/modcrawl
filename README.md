<div align="center">

<img src="./web/static/readme/banner.png" alt="Modcrawl banner" />

[![Crates.io](https://img.shields.io/crates/v/modcrawl?style=flat-square)](https://crates.io/crates/modcrawl)
[![docs.rs](https://img.shields.io/docsrs/modcrawl?style=flat-square)](https://docs.rs/modcrawl)
[![npm](https://img.shields.io/npm/v/modcrawl?style=flat-square)](https://www.npmjs.com/package/modcrawl)
[![CI](https://img.shields.io/github/actions/workflow/status/SirCesarium/modcrawl/ci.yml?branch=main&style=flat-square)](https://github.com/SirCesarium/modcrawl/actions)
[![License](https://img.shields.io/github/license/SirCesarium/modcrawl?style=flat-square)](https://github.com/SirCesarium/modcrawl/blob/main/LICENSE)

</div>

Analyze **Minecraft mods and plugins** to find out which loader they belong to, which dependencies they have, and even **inspect their byte code** to find out **why your modpack is crashing**.

## What is `modcrawl`

`modcrawl` is an all-in-one **Minecraft utility** as a **CLI**, a **Rust library**, and an **npm package** designed to **help modpackers** and **server admins** inspect, organize, and debug modpacks without a headache.

It helps you quickly answer the everyday questions that pop up while building or fixing a modpack:

- **Clean up mixed mod loaders:** Easily spot and remove leftover Fabric mods in a NeoForge modpack (like when **moving away from Sinytra Connector**).
- **Separate Client, Server, Both, and Unknown:** Quickly **identify client-only mods** mixed into a **server folder** so you can purge them before launching.
- **Find the culprit behind cryptic crashes:** When a crash log gives you an obfuscated Java method call but doesn't mention which mod broke, trace references directly back to the offending JAR.
- **Catch conflicts before they happen:** Identify duplicate classes or overlapping mixin targets across multiple mods that might collide on startup.
- **Inspect metadata instantly:** Check mod loaders, dependencies, versions, and internal file lists without manually unzipping every JAR.

### Example of `modcrawl`'s output

<div align="center">

![Modcrawl analyzing Create Mod metadata](./web/static/readme/create-metadata.png)

</div>


## Quickstart

```bash
# Detect mod loader (NeoForge, Fabric, Forge, Paper, Velocity, etc.)
modcrawl loader MyMod.jar

# Read mod metadata (ID, version, authors, license, etc.)
modcrawl metadata MyMod.jar

# Analyze dependencies and find hidden or missing requirements
modcrawl dep MyMod.jar

# Classify mods and plugins as client, server, both, or unknown
modcrawl side mods/

# List all .class files inside a JAR with Java version and access flags
modcrawl classes MyMod.jar

# Search constant pools for references to classes, methods, fields, or strings
modcrawl grep "net/minecraft/world" *.jar --quiet

# Extract mixin targets or detect overlapping mixin conflicts across mods
modcrawl mixins MyMod.jar
modcrawl mixins --overlaps mods/

# Find duplicate .class entries across multiple JARs causing classpath conflicts
modcrawl dupes *.jar

# Scan recursively, limit depth, or exclude folders/files on any command
modcrawl dep -r --exclude disabled mods/
modcrawl dupes -r --depth 2 mods/

# Output machine-readable JSON for scripting or piping
modcrawl side mods/ --json | jq '.[] | select(.side == "client")'
```

## CLI Commands

| Command | Aliases | Description | Key Options |
| --- | --- | --- | --- |
| `dep` | `d`, `deps`, `dependencies` | Analyze mod dependencies and find embedded or missing requirements | `--include-jar-in-jar` (`--jij`), `-j`, `--json` |
| `metadata` | `meta`, `md` | Extract mod and plugin metadata (name, ID, version, authors, etc.) | `-j`, `--json` |
| `loader` | `t`, `ty` | Detect mod loader or server plugin platform | `-j`, `--json` |
| `side` | `sd` | Classify mods and plugins as `client`, `server`, `both`, or `unknown` | `-j`, `--json` |
| `classes` | `c`, `cls` | List all `.class` files with Java classfile version and access flags | `--strict`, `-j`, `--json` |
| `grep` | `g`, `search` | Search strings in constant pools of `.class` files | `-q`, `--quiet`, `--strict`, `-j`, `--json` |
| `mixins` | `m`, `mixin` | Extract `@Mixin` targets or detect overlapping conflicts across JARs | `-o`, `--overlaps`, `-q`, `--quiet`, `--strict`, `-j`, `--json` |
| `dupes` | `dp`, `duplicate` | Detect duplicate classes and classpath collisions across multiple JARs | `-b`, `--by-class`, `-c`, `--count`, `-j`, `--json` |

### Common Options

All commands accept files, directories, or standard input (`-`), and share directory scanning options:

- **`-r, --recursive`**: Recursively scan subdirectories for JAR files.
- **`--depth <N>`**: Limit recursion depth (implies `--recursive`; `1` = direct children only).
- **`--exclude <PATTERN>`**: Skip files or directories matching a glob pattern (`*` matches any characters; repeatable).
- **`-j, --json`**: Output structured JSON instead of human-readable tables.
- **Piping / stdin**: Stream a JAR archive directly or pass newline-separated paths (`cat mod.jar | modcrawl loader -` or `find mods/ -name "*.jar" | modcrawl dep`).

## TL;DR - Shut up and let me download

The binaries are uploaded into the [releases](https://github.com/SirCesarium/modcrawl/releases) section of the GitHub repository. You can download the latest version of `modcrawl` for your platform from there.

### Quick downloads

<div align="center">

<a href="https://github.com/SirCesarium/modcrawl/releases/latest/download/modcrawl-windows-x86_64.msi"><img src="./web/static/readme/windows.png" alt="Modcrawl Windows installer"></a>
<a href="https://github.com/SirCesarium/modcrawl/releases/latest/download/modcrawl-linux-amd64.deb"><img src="./web/static/readme/debian.png" alt="Modcrawl Debian package"></a>
<a href="https://github.com/SirCesarium/modcrawl/releases/latest/download/modcrawl-linux-x86_64.rpm"><img src="./web/static/readme/fedora.png" alt="Modcrawl Fedora package"></a>
<a href="https://github.com/SirCesarium/modcrawl/releases/latest/download/modcrawl-macos-arm64"><img src="./web/static/readme/macos.png" alt="Modcrawl macOS binary"></a>

</div>

The Windows, Debian, and Fedora buttons download installers; macOS downloads the standalone binary. If you have an uncommon architecture (like `Linux ARM64`) you can check the [releases](https://github.com/SirCesarium/modcrawl/releases) page.

Or even check the [Installation](#installation) section.

## Supported loaders

These are the loaders that `modcrawl` currently supports.

*Feel free to contribute to the list by opening a **Pull request**!* ❤️

| Loader | Status | Type |
| --- | --- | --- |
| NeoForge | ✅ | Mods |
| Fabric | ✅ | Mods |
| Forge | ✅ | Mods |
| Forge (legacy `<1.12`) | ✅ | Mods |
| Quilt | ✅ | Mods |
| Bukkit | ✅ | Plugins |
| Paper | ✅ | Plugins |
| Bungee | ✅ | Plugins |
| Velocity | ✅ | Plugins |

## Installation

First, **make sure** you have the **Rust Programming Language** installed on your machine, if not, [click here](https://rust-lang.org/).

### Install with cargo

- Just run `cargo install modcrawl` and you're good to go!

### Build from source

- Clone the repository `git clone https://github.com/SirCesarium/modcrawl.git` and navigate to the `modcrawl` directory.
- Install `modcrawl` with `cargo install --path .`.

## Library usage

### Rust

Add the crate with Cargo and use the exported detect, metadata, and dependency APIs:

```bash
cargo add modcrawl
```

```rust
use std::path::Path;

use modcrawl::{analyze, classify, identify, read_metadata};

fn main() -> modcrawl::Result<()> {
    let jar = Path::new("MyMod.jar");

    println!("{}", identify(jar)?);

    let metadata = read_metadata(jar)?;
    println!("{metadata}");
    println!("{}", classify(&metadata));

    print!("{}", analyze(jar, false)?);

    Ok(())
}
```

### npm

The WASM package exposes the same core functionality to JavaScript and TypeScript:

```bash
npm install modcrawl
```

```js
import init, { identify, metadata_json } from "modcrawl";

await init();

const data = new Uint8Array(await (await fetch("/MyMod.jar")).arrayBuffer());
console.log(identify(data));
console.log(metadata_json(data));
```

## Web

There's a **WIP** web version of `modcrawl` that aims to provide all of the features of `modcrawl` in a more user-friendly way and will work completely offline (thanks to `WASM`!). It's still in development (currently there's not even a MVP), so expect some bugs and missing features. Stay tuned in [GitHub Discussions](https://github.com/SirCesarium/modcrawl/discussions) for any major updates :).
  
> [!NOTE]
> There will be structured documentation when the web version reaches a stable state.
>
> The documentation will provide a deep overview of all the commands and the *API Reference* of the **Rust Library** and the **NPM (TypeScript) Library**

### Web setup

- Clone the repository `git clone https://github.com/SirCesarium/modcrawl.git` and navigate to the `modcrawl/web` directory.
- Make sure you have **Bun Runtime** installed on your machine, if not, click [here](https://bun.sh/) to install it.
- Run `bun install --frozen-lockfile` to install the dependencies.
- Run `bun run dev` and navigate to `http://127.0.0.1:5173` in your browser.

## License

This project is licensed under the terms of the MIT license. See [LICENSE](./LICENSE) for details.
