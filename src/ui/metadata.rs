//! `modcrawl metadata` — the key/value mod information table.
//!
//! A clean header naming the loader (colored) and the mod, then a two-column
//! field/value table, then an optional depends table. Each loader branch
//! extracts its own fields; the shared printer below does the rendering.

use super::style::dim;
use super::style::{loader_style, paint};
use super::table::{cell, print_table, styled_cell};
use crate::core::detect_mod_type::{ForgeModFormat, ModType, PluginType};
use crate::core::loaders::{
    bukkit::metadata::BukkitPluginMetadata, fabric::metadata::FabricModMetadata,
    forge_legacy::metadata::ForgeLegacyModEntry, forge_modern::metadata::ModsTomlMetadata,
    paper::metadata::PaperPluginMetadata,
};
use crate::core::metadata::ModMetadata;
use serde_json::Value;

/// A single labeled field row.
struct Field(&'static str, String);

/// Everything one loader's metadata renders as.
struct Block {
    kind: ModType,
    title: String,
    fields: Vec<Field>,
    depends: Vec<(String, String)>,
}

/// Render one JAR's metadata.
pub(crate) fn render(meta: &ModMetadata) {
    let block = match meta {
        ModMetadata::Fabric(m) => fabric(m),
        ModMetadata::ForgeLegacy(entries) => forge_legacy(entries),
        ModMetadata::ForgeModern(m) => forge_modern(m, ModType::Forge(ForgeModFormat::ModsToml)),
        ModMetadata::NeoForge(m) => forge_modern(m, ModType::NeoForge),
        ModMetadata::Bukkit(m) => bukkit(m),
        ModMetadata::Paper(m) => paper(m),
    };

    print_block(&block);
}

/// Print the header, field table, and optional depends table for a block.
fn print_block(block: &Block) {
    let title = if block.title.is_empty() {
        block.kind.to_string()
    } else {
        format!("{} {}", block.kind, block.title)
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
        let rows: Vec<_> = block
            .depends
            .iter()
            .map(|(name, version)| vec![cell(name), styled_cell(dim(), version)])
            .collect();

        print_table(&["DEPENDS", "VERSION"], &rows);
    }
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
fn fabric(m: &FabricModMetadata) -> Block {
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
            "Homepage",
            contact.homepage.clone().unwrap_or_default(),
        );
    }

    let title = m.name.clone().unwrap_or_else(|| m.id.clone());

    Block {
        kind: ModType::Fabric,
        title,
        fields,
        depends: fabric_depends(m),
    }
}

/// Fabric's `depends` map rendered as name/version pairs.
fn fabric_depends(m: &FabricModMetadata) -> Vec<(String, String)> {
    m.depends
        .iter()
        .map(|(name, range)| (name.clone(), version_string(range)))
        .collect()
}

/// Forge/NeoForge `mods.toml`.
fn forge_modern(m: &ModsTomlMetadata, kind: ModType) -> Block {
    let first = m.mods.first();

    let mut fields = Vec::new();

    push_field(
        &mut fields,
        "Mod ID",
        first.map_or_else(String::new, |e| e.mod_id.clone()),
    );
    push_field(
        &mut fields,
        "Version",
        first.and_then(|e| e.version.clone()).unwrap_or_default(),
    );

    push_field(
        &mut fields,
        "Description",
        first
            .and_then(|e| e.description.clone())
            .unwrap_or_default(),
    );

    let authors = first
        .and_then(|e| e.authors.as_ref())
        .map_or_else(String::new, version_string);
    push_field(&mut fields, "Authors", authors);

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

    let title = first
        .and_then(|e| e.display_name.clone())
        .unwrap_or_default();

    Block {
        kind,
        title,
        fields,
        depends: forge_modern_depends(m),
    }
}

/// Forge/NeoForge `dependencies` map rendered as name/version pairs.
fn forge_modern_depends(m: &ModsTomlMetadata) -> Vec<(String, String)> {
    m.dependencies
        .values()
        .flat_map(|deps| {
            deps.iter().map(|d| {
                let version = d.version_range.clone().unwrap_or_default();
                (d.mod_id.clone(), version)
            })
        })
        .collect()
}

/// Legacy Forge `mcmod.info`.
fn forge_legacy(entries: &[ForgeLegacyModEntry]) -> Block {
    let first = entries.first();

    let mut fields = Vec::new();

    push_field(
        &mut fields,
        "Mod ID",
        first.map_or_else(String::new, |e| e.modid.clone()),
    );
    push_field(
        &mut fields,
        "Name",
        first.and_then(|e| e.name.clone()).unwrap_or_default(),
    );
    push_field(
        &mut fields,
        "Version",
        first.and_then(|e| e.version.clone()).unwrap_or_default(),
    );

    push_field(
        &mut fields,
        "MC Version",
        first.and_then(|e| e.mcversion.clone()).unwrap_or_default(),
    );

    push_field(
        &mut fields,
        "Description",
        first.map_or_else(String::new, |e| e.description.clone().unwrap_or_default()),
    );

    let authors = first.map_or_else(String::new, |e| {
        if e.author_list.is_empty() {
            e.authors.clone().join(", ")
        } else {
            e.author_list.join(", ")
        }
    });
    push_field(&mut fields, "Authors", authors);

    let title = first.and_then(|e| e.name.clone()).unwrap_or_default();

    Block {
        kind: ModType::Forge(ForgeModFormat::McmodInfo),
        title,
        fields,
        depends: Vec::new(),
    }
}

/// Bukkit/Spigot `plugin.yml`.
fn bukkit(m: &BukkitPluginMetadata) -> Block {
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

    Block {
        kind: ModType::Plugin(PluginType::Bukkit),
        title: m.name.clone(),
        fields,
        depends: plugin_depends(&m.depend, &m.softdepend),
    }
}

/// Paper's `paper-plugin.yml`.
fn paper(m: &PaperPluginMetadata) -> Block {
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

    Block {
        kind: ModType::Plugin(PluginType::Paper),
        title: m.name.clone(),
        fields,
        depends: Vec::new(),
    }
}

/// Combine a plugin's hard and soft dependencies into name/version pairs.
fn plugin_depends(hard: &[String], soft: &[String]) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = hard
        .iter()
        .map(|name| (name.clone(), "required".to_owned()))
        .collect();

    pairs.extend(
        soft.iter()
            .map(|name| (name.clone(), "optional".to_owned())),
    );

    pairs
}
