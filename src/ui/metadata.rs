//! `modcrawl metadata` — the key/value mod information table.

use anstream::println;

use super::style::{dim, kind_label, kind_style, loader_name, loader_style, paint};
use super::table::{Cell, cell, print_table, styled_cell};
use crate::core::dep::types::{DepKind, VersionRange};
use crate::core::detect_mod_type::{ForgeModFormat, ModType, PluginType};
use crate::core::loaders::forge_modern::dep::forge_dep_kind;
use crate::core::loaders::{
    bukkit::metadata::BukkitPluginMetadata,
    bungee::metadata::BungeePluginMetadata,
    fabric::metadata::FabricModMetadata,
    forge_legacy::metadata::ForgeLegacyModEntry,
    forge_modern::metadata::{ModsTomlMetadata, ModsTomlMod},
    paper::metadata::PaperPluginMetadata,
    quilt::metadata::QuiltModMetadata,
    velocity::metadata::VelocityPluginMetadata,
};
use crate::core::metadata::ModMetadata;
use serde_json::Value;

/// A single labeled field row.
struct Field(&'static str, String);

/// One row of the depends table. `kind` and `side` are optional; only
/// `mods.toml` records a side.
struct DepRow {
    name: String,
    version: String,
    kind: Option<DepKind>,
    side: Option<String>,
}

/// Everything one loader's metadata renders as.
struct Block {
    kind: ModType,
    title: String,
    fields: Vec<Field>,
    depends: Vec<DepRow>,
}

/// Render one JAR's metadata. `mods.toml` and `mcmod.info` can declare several
/// entries in one file; each becomes its own block.
pub(crate) fn render(meta: &ModMetadata) {
    let blocks = match meta {
        ModMetadata::Fabric(m) => fabric(m),
        ModMetadata::Quilt(m) => quilt(m),
        ModMetadata::ForgeLegacy(entries) => forge_legacy(entries),
        ModMetadata::ForgeModern(m) => forge_modern(m, ModType::Forge(ForgeModFormat::ModsToml)),
        ModMetadata::NeoForge(m) => forge_modern(m, ModType::NeoForge),
        ModMetadata::Bukkit(m) => bukkit(m),
        ModMetadata::Bungee(m) => bungee(m),
        ModMetadata::Paper(m) => paper(m),
        ModMetadata::Velocity(m) => velocity(m),
    };

    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            println!();
        }
        print_block(block);
    }
}

/// Print the header, field table, and optional depends table for a block.
fn print_block(block: &Block) {
    let name = loader_name(&block.kind);
    let title = if block.title.is_empty() {
        name.to_owned()
    } else {
        format!("{} {}", name, block.title)
    };

    println!("{}", paint(loader_style(&block.kind), &title));
    println!();

    if !block.fields.is_empty() {
        let rows: Vec<_> = block
            .fields
            .iter()
            .map(|Field(label, value)| vec![styled_cell(dim(), label), cell(value)])
            .collect();

        print_table(&["FIELD", "VALUE"], &rows);
    }

    if !block.depends.is_empty() {
        println!();
        print_depends(&block.depends);
    }
}

/// Print the depends table, adding KIND and SIDE only when some row has them.
fn print_depends(depends: &[DepRow]) {
    let show_kind = depends.iter().any(|d| d.kind.is_some());
    let show_side = depends.iter().any(|d| d.side.is_some());

    let mut headers = vec!["DEPENDS", "VERSION"];
    if show_kind {
        headers.push("KIND");
    }
    if show_side {
        headers.push("SIDE");
    }

    let rows: Vec<Vec<Cell>> = depends
        .iter()
        .map(|dep| {
            let mut row = vec![cell(&dep.name), styled_cell(dim(), &dep.version)];

            if show_kind {
                row.push(match dep.kind {
                    Some(k) => styled_cell(kind_style(k), kind_label(k)),
                    None => cell(""),
                });
            }

            if show_side {
                row.push(styled_cell(dim(), dep.side.as_deref().unwrap_or("both")));
            }

            row
        })
        .collect();

    print_table(&headers, &rows);
}

/// Append `(label, value)` only when the value is non-empty.
fn push_field(fields: &mut Vec<Field>, label: &'static str, value: String) {
    if !value.is_empty() {
        fields.push(Field(label, value));
    }
}

/// Join authors into a single comma-separated line.
fn join_authors(authors: &[String]) -> String {
    authors.join(", ")
}

/// Map a serde `Value` into a displayable author name.
fn author_name(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Object(map) => map.get("name").and_then(Value::as_str).map(str::to_owned),
        _ => None,
    }
}

/// Flatten a serde `Value` into a version-range string.
fn version_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" or "),
        _ => String::new(),
    }
}

/// Fabric's `fabric.mod.json`.
fn fabric(m: &FabricModMetadata) -> Vec<Block> {
    let mut fields = Vec::new();

    push_field(&mut fields, "ID", m.id.clone());
    push_field(&mut fields, "Version", m.version.clone());
    push_field(
        &mut fields,
        "Description",
        m.description.clone().unwrap_or_default(),
    );

    let authors: Vec<String> = m.authors.iter().filter_map(author_name).collect();
    push_field(&mut fields, "Authors", join_authors(&authors));

    push_field(
        &mut fields,
        "License",
        m.license.clone().unwrap_or_default(),
    );
    push_field(
        &mut fields,
        "Environment",
        m.environment.clone().unwrap_or_default(),
    );

    if let Some(contact) = &m.contact {
        push_field(
            &mut fields,
            "Sources",
            contact.sources.clone().unwrap_or_default(),
        );
        push_field(
            &mut fields,
            "Homepage",
            contact.homepage.clone().unwrap_or_default(),
        );
    }

    let title = m.name.clone().unwrap_or_else(|| m.id.clone());

    vec![Block {
        kind: ModType::Fabric,
        title,
        fields,
        depends: fabric_depends(m),
    }]
}

/// Fabric's `depends` map as rows.
fn fabric_depends(m: &FabricModMetadata) -> Vec<DepRow> {
    m.depends
        .iter()
        .map(|(name, range)| DepRow {
            name: name.clone(),
            version: version_string(range),
            kind: Some(DepKind::Required),
            side: None,
        })
        .collect()
}

/// Quilt's `quilt.mod.json`.
fn quilt(m: &QuiltModMetadata) -> Vec<Block> {
    let loader = &m.quilt_loader;
    let meta = &loader.metadata;
    let mut fields = Vec::new();

    push_field(&mut fields, "ID", loader.id.clone());
    push_field(&mut fields, "Name", meta.name.clone().unwrap_or_default());
    push_field(
        &mut fields,
        "Version",
        loader.version.clone().unwrap_or_default(),
    );
    push_field(
        &mut fields,
        "Description",
        meta.description.clone().unwrap_or_default(),
    );
    push_field(
        &mut fields,
        "License",
        meta.license.as_ref().map_or_else(String::new, |l| l.join()),
    );

    let contributors: Vec<String> = meta
        .contributors
        .iter()
        .map(|(name, role)| {
            if role.is_empty() {
                name.clone()
            } else {
                format!("{name} ({role})")
            }
        })
        .collect();
    push_field(&mut fields, "Contributors", contributors.join(", "));

    if let Some(contact) = &meta.contact {
        push_field(
            &mut fields,
            "Sources",
            contact.sources.clone().unwrap_or_default(),
        );
        push_field(
            &mut fields,
            "Homepage",
            contact.homepage.clone().unwrap_or_default(),
        );
    }

    let depends = loader
        .depends
        .iter()
        .map(|d| DepRow {
            name: d.id().to_owned(),
            version: d.versions().map_or_else(String::new, |v| v.join()),
            kind: Some(DepKind::Required),
            side: None,
        })
        .collect();

    vec![Block {
        kind: ModType::Quilt,
        title: meta.name.clone().unwrap_or_else(|| loader.id.clone()),
        fields,
        depends,
    }]
}

/// Forge/NeoForge `mods.toml`. File-level fields ride on the first block, each
/// `mods[]` entry gets its own, and the merged dependency table goes on the last.
fn forge_modern(m: &ModsTomlMetadata, kind: ModType) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut entries = m.mods.iter();

    if let Some(first) = entries.next() {
        let mut fields = Vec::new();

        push_field(&mut fields, "Loader", loader_name(&kind).to_owned());
        push_field(&mut fields, "Loader Version", loader_version(m, &kind));
        push_field(
            &mut fields,
            "License",
            m.license.clone().unwrap_or_default(),
        );
        push_field(
            &mut fields,
            "Display URL",
            m.display_url.clone().unwrap_or_default(),
        );

        push_mod_fields(&mut fields, first);

        blocks.push(Block {
            kind: kind.clone(),
            title: mod_title(first),
            fields,
            depends: Vec::new(),
        });
    }

    for entry in entries {
        let mut fields = Vec::new();
        push_mod_fields(&mut fields, entry);

        blocks.push(Block {
            kind: kind.clone(),
            title: mod_title(entry),
            fields,
            depends: Vec::new(),
        });
    }

    let depends = forge_modern_depends(m);

    if let Some(last) = blocks.last_mut() {
        last.depends = depends;
    } else {
        // No `mods[]` entries.
        let mut fields = Vec::new();

        push_field(&mut fields, "Loader", loader_name(&kind).to_owned());
        push_field(&mut fields, "Loader Version", loader_version(m, &kind));
        push_field(
            &mut fields,
            "License",
            m.license.clone().unwrap_or_default(),
        );
        push_field(
            &mut fields,
            "Display URL",
            m.display_url.clone().unwrap_or_default(),
        );

        blocks.push(Block {
            kind,
            title: String::new(),
            fields,
            depends,
        });
    }

    blocks
}

/// The loader's own version as a bare number: the lower bound of the `forge` or
/// `neoforge` dependency, falling back to `loaderVersion` verbatim.
fn loader_version(m: &ModsTomlMetadata, kind: &ModType) -> String {
    let loader_id = match kind {
        ModType::NeoForge => "neoforge",
        ModType::Forge(_) => "forge",
        _ => "",
    };

    let from_dep = m
        .dependencies
        .values()
        .flatten()
        .filter(|d| d.mod_id == loader_id)
        .filter_map(|d| d.version_range.clone())
        .find_map(|raw| VersionRange::parse(Some(raw)).min);

    from_dep.unwrap_or_else(|| m.loader_version.clone().unwrap_or_default())
}

/// The per-entry fields every `mods[]` block carries.
fn push_mod_fields(fields: &mut Vec<Field>, entry: &ModsTomlMod) {
    push_field(fields, "Mod ID", entry.mod_id.clone());
    push_field(fields, "Version", entry.version.clone().unwrap_or_default());
    push_field(
        fields,
        "Description",
        entry.description.clone().unwrap_or_default(),
    );

    let authors = entry
        .authors
        .as_ref()
        .map_or_else(String::new, version_string);
    push_field(fields, "Authors", authors);
}

/// A `mods[]` entry's display title: its name, falling back to the mod ID.
fn mod_title(entry: &ModsTomlMod) -> String {
    entry
        .display_name
        .clone()
        .unwrap_or_else(|| entry.mod_id.clone())
}

/// Forge/NeoForge `dependencies` map as rows. The only loader that records a
/// per-dependency side.
fn forge_modern_depends(m: &ModsTomlMetadata) -> Vec<DepRow> {
    m.dependencies
        .values()
        .flat_map(|deps| {
            deps.iter().map(|d| DepRow {
                name: d.mod_id.clone(),
                version: d.version_range.clone().unwrap_or_default(),
                kind: Some(forge_dep_kind(d.dep_type.as_deref())),
                side: d.side.as_ref().map(|s| s.to_lowercase()),
            })
        })
        .collect()
}

/// Legacy Forge `mcmod.info`. The file is an array, so every entry is a block.
fn forge_legacy(entries: &[ForgeLegacyModEntry]) -> Vec<Block> {
    entries
        .iter()
        .map(|entry| {
            let mut fields = Vec::new();

            push_field(&mut fields, "Mod ID", entry.modid.clone());
            push_field(&mut fields, "Name", entry.name.clone().unwrap_or_default());
            push_field(
                &mut fields,
                "Version",
                entry.version.clone().unwrap_or_default(),
            );

            push_field(
                &mut fields,
                "MC Version",
                entry.mcversion.clone().unwrap_or_default(),
            );

            push_field(
                &mut fields,
                "Description",
                entry.description.clone().unwrap_or_default(),
            );

            let authors = if entry.author_list.is_empty() {
                entry.authors.clone().join(", ")
            } else {
                entry.author_list.join(", ")
            };
            push_field(&mut fields, "Authors", authors);

            let title = entry.name.clone().unwrap_or_else(|| entry.modid.clone());

            Block {
                kind: ModType::Forge(ForgeModFormat::McmodInfo),
                title,
                fields,
                depends: legacy_depends(entry),
            }
        })
        .collect()
}

/// A legacy entry's `dependencies` list as rows. Records neither version nor
/// side.
fn legacy_depends(entry: &ForgeLegacyModEntry) -> Vec<DepRow> {
    entry
        .dependencies
        .iter()
        .map(|name| DepRow {
            name: name.clone(),
            version: String::new(),
            kind: Some(DepKind::Required),
            side: None,
        })
        .collect()
}

/// Bukkit/Spigot `plugin.yml`.
fn bukkit(m: &BukkitPluginMetadata) -> Vec<Block> {
    let mut fields = Vec::new();

    push_field(&mut fields, "Name", m.name.clone());
    push_field(&mut fields, "Version", m.version.clone());
    push_field(&mut fields, "Main", m.main.clone());
    push_field(
        &mut fields,
        "API Version",
        m.api_version.clone().unwrap_or_default(),
    );

    push_field(
        &mut fields,
        "Description",
        m.description.clone().unwrap_or_default(),
    );

    let authors = if m.authors.is_empty() {
        m.author.clone().unwrap_or_default()
    } else {
        m.authors.join(", ")
    };
    push_field(&mut fields, "Authors", authors);
    push_field(
        &mut fields,
        "Website",
        m.website.clone().unwrap_or_default(),
    );

    vec![Block {
        kind: ModType::Plugin(PluginType::Bukkit),
        title: m.name.clone(),
        fields,
        depends: plugin_depends(m),
    }]
}

/// Bungee's `bungee.yml`.
fn bungee(m: &BungeePluginMetadata) -> Vec<Block> {
    let mut fields = Vec::new();

    push_field(&mut fields, "Name", m.name.clone());
    push_field(&mut fields, "Version", m.version.clone());
    push_field(&mut fields, "Main", m.main.clone());
    push_field(
        &mut fields,
        "Description",
        m.description.clone().unwrap_or_default(),
    );

    let authors = if m.authors.is_empty() {
        m.author.clone().unwrap_or_default()
    } else {
        m.authors.join(", ")
    };
    push_field(&mut fields, "Authors", authors);

    let depends = m
        .depends
        .iter()
        .map(|name| DepRow {
            name: name.clone(),
            version: String::new(),
            kind: Some(DepKind::Required),
            side: None,
        })
        .collect();

    vec![Block {
        kind: ModType::Plugin(PluginType::Bungee),
        title: m.name.clone(),
        fields,
        depends,
    }]
}

/// Paper's `paper-plugin.yml`.
fn paper(m: &PaperPluginMetadata) -> Vec<Block> {
    let mut fields = Vec::new();

    push_field(&mut fields, "Name", m.name.clone());
    push_field(&mut fields, "Version", m.version.clone());
    push_field(&mut fields, "Main", m.main.clone());
    push_field(
        &mut fields,
        "API Version",
        m.api_version.clone().unwrap_or_default(),
    );

    push_field(
        &mut fields,
        "Description",
        m.description.clone().unwrap_or_default(),
    );

    let authors = if m.authors.is_empty() {
        m.author.clone().unwrap_or_default()
    } else {
        m.authors.join(", ")
    };
    push_field(&mut fields, "Authors", authors);
    push_field(
        &mut fields,
        "Website",
        m.website.clone().unwrap_or_default(),
    );
    push_field(&mut fields, "Load", m.load.clone().unwrap_or_default());

    vec![Block {
        kind: ModType::Plugin(PluginType::Paper),
        title: m.name.clone(),
        fields,
        depends: Vec::new(),
    }]
}

/// Velocity's `velocity-plugin.json`.
fn velocity(m: &VelocityPluginMetadata) -> Vec<Block> {
    let mut fields = Vec::new();

    push_field(&mut fields, "ID", m.id.clone());
    push_field(&mut fields, "Name", m.name.clone().unwrap_or_default());
    push_field(
        &mut fields,
        "Version",
        m.version.clone().unwrap_or_default(),
    );
    push_field(&mut fields, "Main", m.main.clone().unwrap_or_default());
    push_field(
        &mut fields,
        "Description",
        m.description.clone().unwrap_or_default(),
    );
    push_field(&mut fields, "Authors", join_authors(&m.authors));

    let depends = m
        .dependencies
        .iter()
        .map(|d| DepRow {
            name: d.id.clone(),
            version: String::new(),
            kind: Some(if d.optional {
                DepKind::Optional
            } else {
                DepKind::Required
            }),
            side: None,
        })
        .collect();

    vec![Block {
        kind: ModType::Plugin(PluginType::Velocity),
        title: m.name.clone().unwrap_or_else(|| m.id.clone()),
        fields,
        depends,
    }]
}

/// A plugin's hard, soft, and load-before dependencies as rows.
fn plugin_depends(m: &BukkitPluginMetadata) -> Vec<DepRow> {
    let mut rows: Vec<DepRow> = Vec::new();

    rows.extend(m.depend.iter().map(|name| DepRow {
        name: name.clone(),
        version: String::new(),
        kind: Some(DepKind::Required),
        side: None,
    }));

    rows.extend(m.softdepend.iter().map(|name| DepRow {
        name: name.clone(),
        version: String::new(),
        kind: Some(DepKind::Optional),
        side: None,
    }));

    rows.extend(m.loadbefore.iter().map(|name| DepRow {
        name: name.clone(),
        version: String::new(),
        kind: Some(DepKind::LoadBefore),
        side: None,
    }));

    rows
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use toml::from_str;

    #[test]
    fn loader_version_prefers_the_loader_dependency() {
        let m: ModsTomlMetadata = from_str(
            r#"
loaderVersion = "[0,)"

[[mods]]
modId = "create"

[[dependencies.create]]
modId = "neoforge"
versionRange = "[21.1.219,)"
"#,
        )
        .expect("toml parses");

        let v = loader_version(&m, &ModType::NeoForge);
        assert_eq!(v, "21.1.219");
    }

    #[test]
    fn loader_version_uses_the_forge_dependency_for_forge() {
        let m: ModsTomlMetadata = from_str(
            r#"
loaderVersion = "[47,)"

[[mods]]
modId = "forgedemo"

[[dependencies.forgedemo]]
modId = "forge"
versionRange = "[47,)"
"#,
        )
        .expect("toml parses");

        assert_eq!(
            loader_version(&m, &ModType::Forge(ForgeModFormat::ModsToml)),
            "47"
        );
    }

    #[test]
    fn loader_version_falls_back_to_loader_version_field() {
        let m: ModsTomlMetadata = from_str(
            r#"
loaderVersion = "[4,)"

[[mods]]
modId = "nodep"
"#,
        )
        .expect("toml parses");

        assert_eq!(loader_version(&m, &ModType::NeoForge), "[4,)");
    }
}
