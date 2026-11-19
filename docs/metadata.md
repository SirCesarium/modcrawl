---
title: "Metadata"
description: "What a mod or plugin says about itself."
topics:
  - "CLI"
  - "Metadata"
section: "Analysis"
---

`metadata` reads the file each loader uses to describe its mod or plugin, and
prints the fields it finds.

## Usage

```bash
modcrawl metadata MyMod.jar
modcrawl metadata mods/ --json
```

One source prints a single block. With more than one source, each block is
preceded by the path of its file. The block itself starts with the loader label
and the name the mod declares.

## Reading a block

```bash
modcrawl metadata appleskin-fabric-mc1.20.1-2.5.2.jar
```

```text
Fabric AppleSkin

FIELD        VALUE
ID           appleskin
Version      2.5.2+mc1.20.1
Description  Adds various food-related HUD improvements
Authors      squeek502
License      Unlicense
Environment  *
Sources      https://github.com/squeek502/AppleSkin
Homepage     https://minecraft.curseforge.com/projects/appleskin

DEPENDS       VERSION    KIND
fabric-api    *          required
fabricloader  >=0.14.21  required
```

The `DEPENDS` table lists what the loader declares, with the version range and
whether the dependency is required or optional.

## Labels change with the loader

There is no normalized schema in the text output: each loader keeps its own
field names, so the same concept can appear under different labels.

| Concept | Fabric | Quilt | Forge and NeoForge | Forge (legacy) | Plugins |
| --- | --- | --- | --- | --- | --- |
| ID | `ID` | `ID` | `Mod ID` | `Mod ID` | `ID` (Velocity), `Name` (others) |
| Name | header only | `Name` | header only | `Name` | `Name` |
| Authors | `Authors` | `Contributors` | `Authors` | `Authors` | `Authors` |
| Side | `Environment` | `Environment` | `Client side only` | — | — |
| License | `License` | `License` | `License` | — | — |
| Loader version | — | — | `Loader`, `Loader Version` | — | — |
| Entry point | — | — | — | — | `Main` |

Plugins also print fields of their own, such as `API Version` and `Website` for
Bukkit, or `Load` for Paper. `Client side only` appears only when the Forge mod
declares it.

## JSON

`--json` prints a flat object: `"type"` carries the loader id, and the fields of
that loader sit at the top level. The keys come from the loader, not from a
normalized schema.

```bash
modcrawl metadata Chunky-Bukkit-1.5.66.jar --json
```

```json
{
  "type": "bukkit",
  "api-version": "1.13",
  "author": null,
  "authors": ["pop4959"],
  "depend": [],
  "description": "Pre-generates chunks, quickly, efficiently, and safely",
  "folia-supported": true,
  "load": "POSTWORLD",
  "loadbefore": [],
  "main": "org.popcraft.chunky.ChunkyBukkit",
  "name": "Chunky",
  "prefix": null,
  "softdepend": ["dynmap", "BlueMap", "squaremap", "WorldBorder"],
  "version": "1.5.66",
  "website": "https://www.spigotmc.org/resources/chunky.81534/"
}
```

The id is the same value that `loader --json` reports: `fabric`, `quilt`,
`forge`, `forge_legacy`, `neoforge`, `bukkit`, `paper`, `bungee`, or `velocity`.

Legacy Forge is the exception. `mcmod.info` describes a list of mods, so the
object wraps them under `mods`:

```json
{
  "type": "forge_legacy",
  "mods": [
    {
      "modid": "appleskin",
      "name": "AppleSkin",
      "version": "1.0.14",
      "..."
    }
  ]
}
```

The snippet is trimmed; real entries carry the remaining `mcmod.info` fields.

Several files produce a JSON array instead of a single object.

## JARs without metadata

A JAR with no loader metadata, and no embedded JAR that carries some, fails for
that file, and the command exits with status 1:

```text
$ modcrawl metadata library.jar
library.jar: Unsupported metadata format for library.jar
```

With several files, the failure does not stop the run: the remaining files are
still printed, and the exit status reports that something failed.

## Shell JARs

JarJar-style shells keep the real mod inside an embedded JAR. When the outer
archive has no metadata of its own, `metadata` reads the first embedded JAR that
carries some, so the block describes the inner mod.

<!-- TODO: link to /docs/loader and /docs/dep once those pages exist. -->
