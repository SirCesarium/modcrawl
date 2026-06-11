# modcrawl

[![Crates.io](https://img.shields.io/crates/v/modcrawl?style=flat-square)](https://crates.io/crates/modcrawl)
[![CI](https://img.shields.io/github/actions/workflow/status/SirCesarium/modcrawl/ci.yml?branch=main&style=flat-square)](https://github.com/SirCesarium/modcrawl/actions)
[![License](https://img.shields.io/github/license/SirCesarium/modcrawl?style=flat-square)]()

Detect, inspect, and analyze Minecraft mods and plugins from JARs.

```bash
modcrawl type MyMod.jar                           # → Fabric
modcrawl metadata MyMod.jar                       # mod name, version, authors, license
modcrawl metadata MyMod.jar --json                # same, as JSON
modcrawl deps MyMod.jar                           # dependencies with version ranges
modcrawl deps MyMod.jar --include-jar-in-jar       # include embedded JARs
modcrawl classes MyMod.jar                        # list .class files
modcrawl grep "func_" MyMod.jar                   # search constant pool
modcrawl mixins MyMod.jar                         # extract @Mixin targets
modcrawl dupes a.jar b.jar c.jar                  # find duplicate classes
```

```bash
cat *.jar | modcrawl type                         # detect many at once
modcrawl deps *.jar                               # per-file, continues on error
modcrawl grep "net/minecraft/world" *.jar --quiet  # which mods reference a class
modcrawl mixins *.jar --quiet                      # which jars have mixins
modcrawl dupes *.jar --json                        # JSON of all class dupes
```

## How it works

Inspects the ZIP entries inside a JAR without extracting. Picks the first matching sentinel file (Paper > Bukkit > NeoForge > Forge mods.toml > Forge mcmod.info > Fabric), then parses the corresponding metadata file and extracts dependency declarations.

## Supported loaders

| Loader | Sentinel file | Metadata format |
|--------|---------------|-----------------|
| Fabric | `fabric.mod.json` | JSON |
| Forge (mods.toml) | `META-INF/mods.toml` | TOML |
| NeoForge | `META-INF/neoforge.mods.toml` | TOML |
| Forge (mcmod.info) | `mcmod.info` | JSON |
| Bukkit / Spigot | `plugin.yml` | YAML |
| Paper | `paper-plugin.yml` | YAML |

Adding a new loader means one new module implementing `ModHandler` and a single `register()` call — no core changes.

## Install

```bash
cargo install modcrawl
```

Or download a prebuilt binary from [GitHub Releases](https://github.com/SirCesarium/modcrawl/releases) — the `modcrawl.h` header is shipped as a standalone asset (not inside archives).

## Library

```bash
cargo add modcrawl --no-default-features
```

```rust
use modcrawl::core::identify::identify;
use modcrawl::core::metadata::read_metadata;
use modcrawl::core::dep::analyze;

let mod_type = identify("MyMod.jar".as_ref())?;

let meta = read_metadata("MyMod.jar".as_ref())?;
println!("{meta}");                                 // human-readable
println!("{}", serde_json::to_string(&meta)?);       // JSON

let report = analyze("MyMod.jar".as_ref(), false)?;  // exclude jar-in-jar
for dep in &report.dependencies {
    println!("  {} {}", dep.kind.marker(), dep.name);
}
```

Reader variants for in-memory buffers:

```rust
use std::io::Cursor;
use modcrawl::core::identify::identify_reader;
use modcrawl::core::metadata::read_metadata_reader;
use modcrawl::core::dep::analyze_reader;

let data = std::fs::read("MyMod.jar")?;
let mut cursor = Cursor::new(&data[..]);
let mod_type = identify_reader(&mut cursor)?;
```

## Dependency analysis

Dependencies are deduplicated by name. When the same dependency appears with different kinds (e.g. `depends` + `recommends`), the higher-priority kind wins:

Required > LoadBefore > Optional > Recommended > Suggested

Jar-in-jar entries (`META-INF/jars/*.jar`) are always scanned internally. If a dependency matches a bundled JAR's filename, it's removed from the external dependency list. The `--include-jar-in-jar` flag controls whether the jar-in-jar section appears in the output.

## FFI / C bindings

Every core function is exposed as a C-compatible export:

| Function | Returns |
|----------|---------|
| `modcrawl_identify(path)` | Type string |
| `modcrawl_identify_bytes(data, len)` | Type string |
| `modcrawl_metadata(path)` | Human-readable metadata |
| `modcrawl_metadata_json(path)` | JSON metadata |
| `modcrawl_metadata_bytes(data, len)` | Human-readable metadata |
| `modcrawl_metadata_json_bytes(data, len)` | JSON metadata |
| `modcrawl_deps(path, include_jij)` | Human-readable deps |
| `modcrawl_deps_json(path, include_jij)` | JSON deps |
| `modcrawl_deps_bytes(data, len, include_jij)` | Human-readable deps |
| `modcrawl_deps_json_bytes(data, len, include_jij)` | JSON deps |
| `modcrawl_classes_json(path)` | JSON list of .class files |
| `modcrawl_grep_json(path, pattern)` | JSON grep matches |
| `modcrawl_mixins_json(path)` | JSON list of mixin targets |
| `modcrawl_dupes_json(paths)` | JSON list of duplicate classes |
| `modcrawl_free_string(s)` | — (frees returned strings) |

All string-returning functions return null-terminated C strings owned by the caller — free with `modcrawl_free_string`. NULL means an error occurred (details printed to stderr).

```c
#include "modcrawl.h"

char *type = modcrawl_identify("/path/to/MyMod.jar");
if (type) {
    printf("Type: %s\n", type);
    modcrawl_free_string(type);
}

// Metadata as JSON
char *json = modcrawl_metadata_json("/path/to/MyMod.jar");
if (json) {
    printf("%s\n", json);
    modcrawl_free_string(json);
}
```

## CLI reference

| Command | Alias | What |
|---------|-------|------|
| `type` | `t` | Detect mod/plugin type |
| `metadata` | `m` | Read mod metadata (human or `-j` JSON) |
| `dep` | `d`, `deps` | Analyze dependencies (`-j` JSON, `--include-jar-in-jar`/`--jij`) |
| `classes` | `c`, `cls` | List `.class` files with Java version and access flags |
| `grep` | `g`, `search` | Search constant pool strings across all `.class` files |
| `mixins` | `m`, `mixin` | Extract `@Mixin` targets from class-level annotations |
| `dupes` | `dp`, `duplicate` | Find duplicate `.class` entries across multiple JARs |

### `type`

Reads from stdin when given no file arguments. Stdin can be raw ZIP bytes (detects EOCD boundaries for `cat *.jar`) or newline-separated file paths.

### `metadata`

Takes one or more JAR paths. Multi-file output prefixes each result with the filename. `--json` / `-j` for compact JSON (one line per file).

### `dep` / `deps`

Takes one or more JAR paths. `--json` / `-j` for pretty-printed JSON. `--include-jar-in-jar` / `--jij` to show embedded JARs. Bundled dependencies are always filtered out of the external dependency list regardless of this flag.

### `classes` (requires `classfile` feature)

Lists every `.class` file inside a JAR with its Java version and access flags.

```
$ modcrawl classes MyMod.jar
  mezz/jei/JustEnoughItems.class    Java 8   public
  mezz/jei/color/ColorGetter.class  Java 8   public final
```

`--json` / `-j` for machine-readable output.

### `grep` (requires `classfile` feature)

Searches the **constant pool** of every `.class` file inside a JAR — no decompilation needed. Finds class references, method calls, field accesses, annotations, and raw UTF-8 strings.

```
$ modcrawl grep "func_186724_a" MyMod.jar
  mezz/jei/color/ColorGetter.class:
    Utf8: "(Lnet/minecraft/world/IBlockAccess;...)I"
    MethodRef: "net/minecraft/client/renderer/color/BlockColors.func_186724_a:..."

$ modcrawl grep "net/minecraft/world" *.jar --quiet
  jei_1.12.2-4.16.1.1013.jar:
    mezz/jei/color/ColorGetter.class
    mezz/jei/plugins/vanilla/anvil/AnvilRecipeMaker.class
```

Useful for crash investigation: find which mod references an obfuscated function or class without opening a single JAR.

Flags: `--quiet` / `-q` (only show class file names), `--json` / `-j` (JSON output).

### `mixins` (requires `classfile` feature)

Extracts `@Mixin` targets from SpongePowered mixin annotations at the class level. Parses `RuntimeVisibleAnnotations` / `RuntimeInvisibleAnnotations` attributes and resolves both `value` (Class ref) and `targets` (String ref) element pairs — no decompilation needed.

```
$ modcrawl mixins MyMod.jar
  some/mod/mixin/ExampleMixin.class ->
    net/minecraft/world/entity/player/Player
  another/mod/mixin/TitleScreenMixin.class ->
    net/minecraft/client/gui/screens/TitleScreen
```

`--quiet` / `-q` (only show mixin class names), `--json` / `-j` (JSON output).

### `dupes` (requires `classfile` feature)

Finds `.class` files that appear in more than one JAR — useful for detecting classpath conflicts between mods. Uses zip entry listing only, no classfile parsing needed.

By default, output is grouped by conflict pair (two JARs that share classes):

```
$ modcrawl dupes a.jar b.jar c.jar
── Classpath Conflicts ──

  a.jar
  b.jar
  ── 3 shared classes ──
    com/example/SomeClass.class
    com/example/OtherClass.class
    com/example/ThirdClass.class
```

`--by-class` / `-b` groups by class instead (legacy format), showing how many JARs each class appears in:

```
$ modcrawl dupes a.jar b.jar c.jar --by-class
  com/example/SomeClass.class  (in 2 JARs)
    a.jar
    b.jar
```

`--count` / `-c` shows only a summary:

```
$ modcrawl dupes *.jar --count
Found 42 duplicate classes across 15 JAR files.
```

`--json` / `-j` for machine-readable output. Combine with `--count` for a compact JSON summary.

## License

MIT
