//! Class-file inspection: class listings, constant-pool search, mixin targets,
//! and duplicate-class detection.
//!
//! The jars embedded in an archive are part of it: their classes are included
//! too, and every report points at them as `<jar>!/<entry>`.

use std::any::Any;
use std::io::{Cursor, Read, Seek};
use std::panic;
use std::path::Path;

use java_class_rs::{
    ClassAccessFlags, ClassFile, ConstantPoolEntry, ElementValue, ParsedAttribute, get_entry,
    get_utf8, parse_classfile, parse_specialized_attribute,
};
use serde::Serialize;
use zipcrawl::ZipManager;

use crate::core::embedded::visit_embedded;
use crate::core::side::Side;
use crate::error::{Error, Result};

mod dupes;

#[cfg(feature = "cli")]
pub(crate) use dupes::ClassIndex;
pub use dupes::{
    ConflictEntry, DupEntry, find_duplicates, find_duplicates_buffers, group_conflicts,
};

/// A class file found inside a JAR, with its Java version and access flags.
#[derive(Debug, Clone, Serialize)]
pub struct ClassEntry {
    /// Archive-relative path of the `.class` entry.
    pub name: String,
    /// Java class-file version rendered for display.
    pub java_version: String,
    /// Human-readable class access flags.
    pub access_flags: Vec<String>,
}

/// Result of scanning the class files in a JAR.
#[derive(Debug, Clone, Serialize)]
pub struct ClassScan<T> {
    /// Entries collected from class files that parsed successfully.
    pub items: Vec<T>,
    /// Class files that could not be parsed and were skipped.
    pub skipped: Vec<SkippedClass>,
}

/// A class file that was skipped during a scan.
#[derive(Debug, Clone, Serialize)]
pub struct SkippedClass {
    /// Archive-relative path of the `.class` entry.
    pub name: String,
    /// Why the class file could not be parsed.
    pub reason: String,
}

/// A constant-pool value that matched a `grep` pattern.
#[derive(Debug, Clone, Serialize)]
pub struct GrepMatch {
    /// Class file that contains the matching constant.
    pub class_file: String,
    /// Constant-pool tag of the matching entry.
    pub pool_tag: String,
    /// Decoded constant value.
    pub value: String,
}

/// A class-level `@Mixin` annotation and the class it targets.
#[derive(Debug, Clone, Serialize)]
pub struct MixinEntry {
    /// Class that declares the `@Mixin` annotation.
    pub mixin_class: String,
    /// Internal name of the class being mixed into.
    pub target: String,
}

/// A mixin target class that is mixed into by classes from more than one JAR.
#[derive(Debug, Clone, Serialize)]
pub struct MixinOverlap {
    /// Internal name of the targeted class.
    pub target: String,
    /// Every `(jar_label, mixin_class)` pair that targets this class.
    pub mixins: Vec<OverlapEntry>,
}

/// One `(jar, mixin_class)` contributor to a [`MixinOverlap`].
#[derive(Debug, Clone, Serialize)]
pub struct OverlapEntry {
    /// Label of the JAR that contains the mixin class.
    pub jar: String,
    /// Internal name of the mixin class inside that JAR.
    pub mixin_class: String,
}

/// Group `(jar_label, MixinEntry)` pairs by target and return only those
/// targets that are mixed into by classes from more than one JAR.
///
/// Results are sorted by target name; within each target, contributors are
/// sorted by JAR label then by mixin class name.
#[must_use]
pub fn group_mixin_overlaps(entries: &[(String, MixinEntry)]) -> Vec<MixinOverlap> {
    use std::collections::BTreeMap;

    let mut by_target: BTreeMap<&str, Vec<OverlapEntry>> = BTreeMap::new();

    for (jar, entry) in entries {
        by_target
            .entry(entry.target.as_str())
            .or_default()
            .push(OverlapEntry {
                jar: jar.clone(),
                mixin_class: entry.mixin_class.clone(),
            });
    }

    by_target
        .into_iter()
        .filter(|(_, mixins)| {
            let mut jars: Vec<&str> = mixins.iter().map(|e| e.jar.as_str()).collect();
            jars.sort_unstable();
            jars.dedup();
            jars.len() > 1
        })
        .map(|(target, mut mixins)| {
            mixins.sort_by(|a, b| a.jar.cmp(&b.jar).then(a.mixin_class.cmp(&b.mixin_class)));
            MixinOverlap {
                target: target.to_owned(),
                mixins,
            }
        })
        .collect()
}

/// List all `.class` files in a JAR with basic metadata.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn list_classes(path: &Path) -> Result<Vec<ClassEntry>> {
    Ok(scan_classes(path)?.items)
}

/// List all `.class` files in a JAR supplied by a `Read` source.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn list_classes_reader<R: Read>(reader: &mut R) -> Result<Vec<ClassEntry>> {
    Ok(scan_classes_reader(reader)?.items)
}

/// List all `.class` files in a JAR and report the ones that were skipped.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn scan_classes(path: &Path) -> Result<ClassScan<ClassEntry>> {
    let mut manager = ZipManager::new(path)?;
    list_classes_impl(&mut manager)
}

/// List all `.class` files in a JAR supplied by a `Read` source and report the
/// ones that were skipped.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn scan_classes_reader<R: Read>(reader: &mut R) -> Result<ClassScan<ClassEntry>> {
    let mut manager = ZipManager::from_reader(reader)?;
    list_classes_impl(&mut manager)
}

fn list_classes_impl<R: Read + Seek>(manager: &mut ZipManager<R>) -> Result<ClassScan<ClassEntry>> {
    scan_jar(manager, "", &mut |cf, name| {
        vec![ClassEntry {
            name: name.to_string(),
            java_version: java_version_string(cf.major_version, cf.minor_version),
            access_flags: format_class_access_flags(cf.access_flags),
        }]
    })
}

/// Search the constant pool of all `.class` files in a JAR for a pattern.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn grep(path: &Path, pattern: &str) -> Result<Vec<GrepMatch>> {
    Ok(scan_grep(path, pattern)?.items)
}

/// Search the constant pool of all `.class` files in a JAR supplied by a
/// `Read` source.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn grep_reader<R: Read>(reader: &mut R, pattern: &str) -> Result<Vec<GrepMatch>> {
    Ok(scan_grep_reader(reader, pattern)?.items)
}

/// Search the constant pool of all `.class` files in a JAR and report the ones
/// that were skipped.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn scan_grep(path: &Path, pattern: &str) -> Result<ClassScan<GrepMatch>> {
    let mut manager = ZipManager::new(path)?;
    grep_impl(&mut manager, pattern)
}

/// Search the constant pool of all `.class` files in a JAR supplied by a
/// `Read` source and report the ones that were skipped.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn scan_grep_reader<R: Read>(reader: &mut R, pattern: &str) -> Result<ClassScan<GrepMatch>> {
    let mut manager = ZipManager::from_reader(reader)?;
    grep_impl(&mut manager, pattern)
}

fn grep_impl<R: Read + Seek>(
    manager: &mut ZipManager<R>,
    pattern: &str,
) -> Result<ClassScan<GrepMatch>> {
    scan_jar(manager, "", &mut |cf, name| {
        search_constant_pool(cf, name, pattern)
    })
}

/// Extract mixin targets from all `.class` files in a JAR.
///
/// Looks for `@Mixin` (`SpongePowered`) annotations at the class level.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn mixins(path: &Path) -> Result<Vec<MixinEntry>> {
    Ok(scan_mixins(path)?.items)
}

/// Extract mixin targets from a JAR supplied by a `Read` source.
///
/// Looks for `@Mixin` (`SpongePowered`) annotations at the class level.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn mixins_reader<R: Read>(reader: &mut R) -> Result<Vec<MixinEntry>> {
    Ok(scan_mixins_reader(reader)?.items)
}

/// Extract mixin targets from a JAR and report the class files that were
/// skipped.
///
/// Looks for `@Mixin` (`SpongePowered`) annotations at the class level.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn scan_mixins(path: &Path) -> Result<ClassScan<MixinEntry>> {
    let mut manager = ZipManager::new(path)?;
    mixins_impl(&mut manager)
}

/// Extract mixin targets from a JAR supplied by a `Read` source and report the
/// class files that were skipped.
///
/// Looks for `@Mixin` (`SpongePowered`) annotations at the class level.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn scan_mixins_reader<R: Read>(reader: &mut R) -> Result<ClassScan<MixinEntry>> {
    let mut manager = ZipManager::from_reader(reader)?;
    mixins_impl(&mut manager)
}

/// Find overlapping mixin targets across multiple JARs on disk.
///
/// Looks for `@Mixin` targets that appear in more than one JAR.
///
/// # Errors
///
/// Returns an error if any JAR cannot be read.
pub fn find_mixin_overlaps(paths: &[&Path]) -> Result<Vec<MixinOverlap>> {
    let mut entries = Vec::new();
    for &path in paths {
        let label = path.to_string_lossy().into_owned();
        for mixin in mixins(path)? {
            entries.push((label.clone(), mixin));
        }
    }
    Ok(group_mixin_overlaps(&entries))
}

/// Find overlapping mixin targets across in-memory JARs labeled by name.
///
/// Looks for `@Mixin` targets that appear in more than one JAR.
///
/// # Errors
///
/// Returns an error if any buffer is not a valid ZIP archive.
pub fn find_mixin_overlaps_buffers(jars: &[(&str, &[u8])]) -> Result<Vec<MixinOverlap>> {
    let mut entries = Vec::new();
    for &(label, bytes) in jars {
        let mut cursor = Cursor::new(bytes);
        for mixin in mixins_reader(&mut cursor)? {
            entries.push((label.to_owned(), mixin));
        }
    }
    Ok(group_mixin_overlaps(&entries))
}

/// Annotation descriptors that identify a `@Mod` class or a dist-restricting
/// annotation on the main class. Used as fast byte-level pre-filters before
/// the class is parsed.
const MOD_NEEDLES: [&[u8]; 6] = [
    b"Lnet/neoforged/fml/common/Mod;",
    b"Lcpw/mods/fml/common/Mod;",
    b"Lnet/minecraftforge/fml/common/Mod;",
    b"Lnet/minecraftforge/api/distmarker/OnlyIn;",
    b"Lcpw/mods/fml/relauncher/SideOnly;",
    b"Lnet/minecraftforge/fml/relauncher/SideOnly;",
];

/// Scan class-level annotations in the archive and its embedded JARs to infer
/// the side the mod runs on.
///
/// Reads `@Mod(dist = …)` for `NeoForge`, `@Mod(clientSideOnly = …)` and
/// `@Mod(serverSideOnly = …)` for Forge/FML 1.x, and `@OnlyIn`/`@SideOnly`
/// on the main class as a fallback.
///
/// Returns `None` when no `@Mod`-annotated class is found.
///
/// # Errors
///
/// Returns an error if archive entries cannot be read.
pub fn scan_mod_side<R: Read + Seek>(manager: &mut ZipManager<R>) -> Result<Option<Side>> {
    let mut sides = Vec::new();
    scan_archive_mod_side(manager, &mut sides)?;

    visit_embedded(manager, |_path, nested| {
        scan_archive_mod_side(nested, &mut sides)?;
        Ok(())
    })?;

    if sides.is_empty() {
        return Ok(None);
    }

    // Any "both" or a mix of client and server means both.
    if sides.contains(&Side::Both)
        || (sides.contains(&Side::Client) && sides.contains(&Side::Server))
    {
        return Ok(Some(Side::Both));
    }

    if sides.iter().all(|s| *s == Side::Client) {
        return Ok(Some(Side::Client));
    }

    if sides.iter().all(|s| *s == Side::Server) {
        return Ok(Some(Side::Server));
    }

    Ok(Some(Side::Both))
}

/// Constant-pool prefix of a dedicated-server class reference.
const SERVER_CLASS_NEEDLE: &[u8] = b"net/minecraft/server/";

/// Whether any class at the root of the archive references a dedicated-server
/// class.
///
/// A mod whose classes never mention `net/minecraft/server/` cannot run any
/// server-side code of its own. Embedded JARs are ignored on purpose: a bundled
/// library's server references say nothing about the mod shipping it.
///
/// # Errors
///
/// Returns an error if archive entries cannot be read.
pub fn references_server_classes<R: Read + Seek>(manager: &mut ZipManager<R>) -> Result<bool> {
    for entry in manager.entries()? {
        if entry.is_dir || !is_class_entry(&entry.name) {
            continue;
        }

        let file = manager.open_file(&entry.name)?;
        let mut bytes = Vec::new();
        let read = file
            .take((MAX_CLASS_SIZE as u64) + 1)
            .read_to_end(&mut bytes)
            .map_err(Error::Io)?;
        if read > MAX_CLASS_SIZE {
            continue;
        }

        if bytes
            .windows(SERVER_CLASS_NEEDLE.len())
            .any(|window| window == SERVER_CLASS_NEEDLE)
        {
            return Ok(true);
        }
    }

    Ok(false)
}

fn scan_archive_mod_side<R: Read + Seek>(
    manager: &mut ZipManager<R>,
    sides: &mut Vec<Side>,
) -> Result<()> {
    let entries = manager.entries()?;
    for entry in &entries {
        if !is_class_entry(&entry.name) || entry.is_dir {
            continue;
        }

        let file = manager.open_file(&entry.name)?;
        let mut bytes = Vec::new();
        let read = file
            .take((MAX_CLASS_SIZE as u64) + 1)
            .read_to_end(&mut bytes)
            .map_err(Error::Io)?;
        if read > MAX_CLASS_SIZE {
            continue;
        }

        // Skip classes that have no relevant annotation descriptor at all.
        let has_needle = MOD_NEEDLES
            .iter()
            .any(|needle| bytes.windows(needle.len()).any(|w| w == *needle));
        if !has_needle {
            continue;
        }

        let Ok(Ok((_, cf))) = panic::catch_unwind(|| parse_classfile(&bytes)) else {
            continue;
        };

        if let Some(side) = extract_class_mod_side(&cf) {
            sides.push(side);
        }
    }

    Ok(())
}

/// Extract a side from class-level annotations on a single class file.
///
/// Returns `None` when the class has no `@Mod` annotation at all, so the
/// caller can distinguish "no information" from "explicitly both".
fn extract_class_mod_side(cf: &ClassFile) -> Option<Side> {
    let mut is_mod_class = false;
    let mut neoforge_dist: Option<Side> = None;
    let mut forge_client_only = false;
    let mut forge_server_only = false;
    let mut only_in_side: Option<Side> = None;

    for attr in &cf.attributes {
        let parsed = parse_specialized_attribute(attr, &cf.constant_pool);
        let anns = match parsed {
            ParsedAttribute::RuntimeVisibleAnnotations(ref a)
            | ParsedAttribute::RuntimeInvisibleAnnotations(ref a) => &a.annotations,
            _ => continue,
        };

        for ann in anns {
            let Some(ann_type) = resolve_utf8(&cf.constant_pool, ann.type_index) else {
                continue;
            };

            if ann_type == "Lnet/neoforged/fml/common/Mod;" {
                is_mod_class = true;
                for (name_idx, val) in &ann.element_value_pairs {
                    if resolve_utf8(&cf.constant_pool, *name_idx).as_deref() != Some("dist") {
                        continue;
                    }
                    if let ElementValue::Enum {
                        const_name_index, ..
                    } = val
                    {
                        match resolve_utf8(&cf.constant_pool, *const_name_index).as_deref() {
                            Some("CLIENT") => neoforge_dist = Some(Side::Client),
                            Some("DEDICATED_SERVER" | "SERVER") => {
                                neoforge_dist = Some(Side::Server);
                            }
                            _ => {}
                        }
                    }
                }
            } else if ann_type == "Lcpw/mods/fml/common/Mod;"
                || ann_type == "Lnet/minecraftforge/fml/common/Mod;"
            {
                is_mod_class = true;
                for (name_idx, val) in &ann.element_value_pairs {
                    let Some(param) = resolve_utf8(&cf.constant_pool, *name_idx) else {
                        continue;
                    };
                    if param == "clientSideOnly"
                        && let ElementValue::Boolean { const_value_index } = val
                        && let Some(ConstantPoolEntry::Integer(info)) =
                            get_entry(&cf.constant_pool, *const_value_index)
                        && info.value != 0
                    {
                        forge_client_only = true;
                    } else if param == "serverSideOnly"
                        && let ElementValue::Boolean { const_value_index } = val
                        && let Some(ConstantPoolEntry::Integer(info)) =
                            get_entry(&cf.constant_pool, *const_value_index)
                        && info.value != 0
                    {
                        forge_server_only = true;
                    }
                }
            } else if ann_type == "Lnet/minecraftforge/api/distmarker/OnlyIn;"
                || ann_type == "Lcpw/mods/fml/relauncher/SideOnly;"
                || ann_type == "Lnet/minecraftforge/fml/relauncher/SideOnly;"
            {
                for (name_idx, val) in &ann.element_value_pairs {
                    if resolve_utf8(&cf.constant_pool, *name_idx).as_deref() != Some("value") {
                        continue;
                    }
                    if let ElementValue::Enum {
                        const_name_index, ..
                    } = val
                    {
                        match resolve_utf8(&cf.constant_pool, *const_name_index).as_deref() {
                            Some("CLIENT") => only_in_side = Some(Side::Client),
                            Some("DEDICATED_SERVER" | "SERVER") => {
                                only_in_side = Some(Side::Server);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    if !is_mod_class {
        return None;
    }

    if let Some(dist) = neoforge_dist {
        return Some(dist);
    }

    if forge_client_only || only_in_side == Some(Side::Client) {
        return Some(Side::Client);
    }

    if forge_server_only || only_in_side == Some(Side::Server) {
        return Some(Side::Server);
    }

    Some(Side::Both)
}

fn mixins_impl<R: Read + Seek>(manager: &mut ZipManager<R>) -> Result<ClassScan<MixinEntry>> {
    scan_jar(manager, "", &mut extract_mixin_targets)
}

fn extract_mixin_targets(cf: &ClassFile, class_name: &str) -> Vec<MixinEntry> {
    let mut results = Vec::new();

    for attr in &cf.attributes {
        let parsed = parse_specialized_attribute(attr, &cf.constant_pool);
        match parsed {
            ParsedAttribute::RuntimeVisibleAnnotations(ref anns)
            | ParsedAttribute::RuntimeInvisibleAnnotations(ref anns) => {
                for ann in &anns.annotations {
                    let Some(ann_type) = resolve_utf8(&cf.constant_pool, ann.type_index) else {
                        continue;
                    };
                    if !is_mixin_annotation(&ann_type) {
                        continue;
                    }
                    for (name_idx, value) in &ann.element_value_pairs {
                        let Some(elem_name) = resolve_utf8(&cf.constant_pool, *name_idx) else {
                            continue;
                        };
                        if elem_name != "value" && elem_name != "targets" {
                            continue;
                        }
                        match value {
                            ElementValue::Class { class_info_index } => {
                                if let Some(target) =
                                    resolve_class_descriptor(&cf.constant_pool, *class_info_index)
                                {
                                    results.push(MixinEntry {
                                        mixin_class: class_name.to_string(),
                                        target,
                                    });
                                }
                            }
                            ElementValue::String { const_value_index } => {
                                if let Some(target) =
                                    resolve_utf8(&cf.constant_pool, *const_value_index)
                                {
                                    results.push(MixinEntry {
                                        mixin_class: class_name.to_string(),
                                        target,
                                    });
                                }
                            }
                            ElementValue::Array { values } => {
                                for val in values {
                                    match val {
                                        ElementValue::Class { class_info_index } => {
                                            if let Some(target) = resolve_class_descriptor(
                                                &cf.constant_pool,
                                                *class_info_index,
                                            ) {
                                                results.push(MixinEntry {
                                                    mixin_class: class_name.to_string(),
                                                    target,
                                                });
                                            }
                                        }
                                        ElementValue::String { const_value_index } => {
                                            if let Some(target) =
                                                resolve_utf8(&cf.constant_pool, *const_value_index)
                                            {
                                                results.push(MixinEntry {
                                                    mixin_class: class_name.to_string(),
                                                    target,
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    results
}

fn is_mixin_annotation(descriptor: &str) -> bool {
    descriptor == "Lorg/spongepowered/asm/mixin/Mixin;"
}

fn resolve_class_descriptor(cp: &[ConstantPoolEntry], index: u16) -> Option<String> {
    let desc = resolve_utf8(cp, index)?;
    let cleaned = desc
        .strip_prefix('L')
        .and_then(|s| s.strip_suffix(';'))
        .unwrap_or(&desc);
    Some(cleaned.to_string())
}

fn is_class_entry(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("class"))
}

const MAX_CLASS_SIZE: usize = 32 * 1024 * 1024;

/// Whether a class entry was parsed or deliberately skipped.
enum ClassParse {
    Parsed(ClassFile),
    Skipped(String),
}

/// Scan every `.class` entry in `manager` and in the jars embedded in it,
/// collecting what `collect` returns for each parsed class and recording every
/// class that was skipped.
///
/// A class found in an embedded jar is labeled `<jar>!/<entry>` so it stays
/// traceable to the artifact that ships it.
fn scan_jar<R, T, F>(
    manager: &mut ZipManager<R>,
    prefix: &str,
    collect: &mut F,
) -> Result<ClassScan<T>>
where
    R: Read + Seek,
    F: FnMut(&ClassFile, &str) -> Vec<T>,
{
    let mut scan = scan_entries(manager, prefix, collect)?;

    visit_embedded(manager, |path, nested| {
        let embedded = scan_entries(nested, &format!("{prefix}{path}!/"), collect)?;
        scan.items.extend(embedded.items);
        scan.skipped.extend(embedded.skipped);

        Ok(())
    })?;

    Ok(scan)
}

/// Scan the `.class` entries of a single archive level.
fn scan_entries<R, T, F>(
    manager: &mut ZipManager<R>,
    prefix: &str,
    collect: &mut F,
) -> Result<ClassScan<T>>
where
    R: Read + Seek,
    F: FnMut(&ClassFile, &str) -> Vec<T>,
{
    let entries = manager.entries()?;
    let mut items = Vec::new();
    let mut skipped = Vec::new();

    for entry in &entries {
        if !is_class_entry(&entry.name) || entry.is_dir {
            continue;
        }

        let label = format!("{prefix}{}", entry.name);

        match parse_class_in_jar(manager, &entry.name)? {
            ClassParse::Parsed(cf) => items.extend(collect(&cf, &label)),
            ClassParse::Skipped(reason) => skipped.push(SkippedClass {
                name: label,
                reason,
            }),
        }
    }

    Ok(ClassScan { items, skipped })
}

fn parse_class_in_jar<R: Read + Seek>(
    manager: &mut ZipManager<R>,
    name: &str,
) -> Result<ClassParse> {
    let file = manager.open_file(name)?;
    let mut bytes = Vec::new();
    let read = file
        .take((MAX_CLASS_SIZE as u64) + 1)
        .read_to_end(&mut bytes)
        .map_err(Error::Io)?;
    if read > MAX_CLASS_SIZE {
        return Ok(ClassParse::Skipped(format!(
            "exceeds the maximum class size of {MAX_CLASS_SIZE} bytes"
        )));
    }

    match panic::catch_unwind(|| parse_classfile(&bytes)) {
        Ok(Ok((_, cf))) => Ok(ClassParse::Parsed(cf)),
        Ok(Err(err)) => Ok(ClassParse::Skipped(err.to_string())),
        Err(payload) => Ok(ClassParse::Skipped(format!(
            "class parser panicked: {}",
            panic_message(payload.as_ref())
        ))),
    }
}

/// Best-effort text from a caught panic payload.
fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::from("unknown panic")
    }
}

fn search_constant_pool(cf: &ClassFile, class_name: &str, pattern: &str) -> Vec<GrepMatch> {
    let mut results = Vec::new();

    for entry in &cf.constant_pool {
        if let ConstantPoolEntry::Utf8(utf8) = entry
            && utf8.value.contains(pattern)
        {
            results.push(GrepMatch {
                class_file: class_name.to_string(),
                pool_tag: "Utf8".to_string(),
                value: utf8.value.clone(),
            });
        }
    }

    for entry in &cf.constant_pool {
        match entry {
            ConstantPoolEntry::Class(info) => {
                if let Some(name) = resolve_utf8(&cf.constant_pool, info.name_index)
                    && name.contains(pattern)
                {
                    results.push(GrepMatch {
                        class_file: class_name.to_string(),
                        pool_tag: "Class".to_string(),
                        value: name,
                    });
                }
            }
            ConstantPoolEntry::Methodref(info) => {
                if let Some(resolved) = resolve_ref(
                    &cf.constant_pool,
                    info.class_index,
                    info.name_and_type_index,
                ) && resolved.contains(pattern)
                {
                    results.push(GrepMatch {
                        class_file: class_name.to_string(),
                        pool_tag: "MethodRef".to_string(),
                        value: resolved,
                    });
                }
            }
            ConstantPoolEntry::Fieldref(info) => {
                if let Some(resolved) = resolve_ref(
                    &cf.constant_pool,
                    info.class_index,
                    info.name_and_type_index,
                ) && resolved.contains(pattern)
                {
                    results.push(GrepMatch {
                        class_file: class_name.to_string(),
                        pool_tag: "FieldRef".to_string(),
                        value: resolved,
                    });
                }
            }
            ConstantPoolEntry::InterfaceMethodref(info) => {
                if let Some(resolved) = resolve_ref(
                    &cf.constant_pool,
                    info.class_index,
                    info.name_and_type_index,
                ) && resolved.contains(pattern)
                {
                    results.push(GrepMatch {
                        class_file: class_name.to_string(),
                        pool_tag: "InterfaceMethodRef".to_string(),
                        value: resolved,
                    });
                }
            }
            _ => {}
        }
    }

    results
}

fn resolve_utf8(cp: &[ConstantPoolEntry], index: u16) -> Option<String> {
    get_utf8(cp, index).map(String::from)
}

fn resolve_class_name(cp: &[ConstantPoolEntry], class_index: u16) -> Option<String> {
    let entry = get_entry(cp, class_index)?;
    if let ConstantPoolEntry::Class(info) = entry {
        resolve_utf8(cp, info.name_index)
    } else {
        None
    }
}

fn resolve_ref(cp: &[ConstantPoolEntry], class_index: u16, nat_index: u16) -> Option<String> {
    let class_name = resolve_class_name(cp, class_index)?;
    let nat = get_entry(cp, nat_index)?;
    if let ConstantPoolEntry::NameAndType(nat) = nat {
        let name = resolve_utf8(cp, nat.name_index)?;
        let descriptor = resolve_utf8(cp, nat.descriptor_index)?;
        Some(format!("{class_name}.{name}:{descriptor}"))
    } else {
        None
    }
}

fn java_version_string(major: u16, _minor: u16) -> String {
    match major {
        45 => "1.1".to_string(),
        46 => "1.2".to_string(),
        47 => "1.3".to_string(),
        48 => "1.4".to_string(),
        49 => "5".to_string(),
        50 => "6".to_string(),
        51 => "7".to_string(),
        52 => "8".to_string(),
        53 => "9".to_string(),
        54 => "10".to_string(),
        55 => "11".to_string(),
        56 => "12".to_string(),
        57 => "13".to_string(),
        58 => "14".to_string(),
        59 => "15".to_string(),
        60 => "16".to_string(),
        61 => "17".to_string(),
        62 => "18".to_string(),
        63 => "19".to_string(),
        64 => "20".to_string(),
        65 => "21".to_string(),
        m if m > 65 => format!("{}+", m - 44),
        m => format!("{m}"),
    }
}

fn format_class_access_flags(flags: ClassAccessFlags) -> Vec<String> {
    let mut result = Vec::new();
    if flags.contains(ClassAccessFlags::PUBLIC) {
        result.push("public".to_string());
    }
    if flags.contains(ClassAccessFlags::FINAL) {
        result.push("final".to_string());
    }
    if flags.contains(ClassAccessFlags::ABSTRACT) {
        result.push("abstract".to_string());
    }
    if flags.contains(ClassAccessFlags::INTERFACE) {
        result.push("interface".to_string());
    }
    if flags.contains(ClassAccessFlags::ANNOTATION) {
        result.push("annotation".to_string());
    }
    if flags.contains(ClassAccessFlags::ENUM) {
        result.push("enum".to_string());
    }
    if flags.contains(ClassAccessFlags::SYNTHETIC) {
        result.push("synthetic".to_string());
    }
    if flags.contains(ClassAccessFlags::MODULE) {
        result.push("module".to_string());
    }
    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::{Cursor, Write};

    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;

    /// Minimal valid class file: one `Utf8` and one `Class` entry named `A`.
    const VALID_CLASS: &[u8] = &[
        0xca, 0xfe, 0xba, 0xbe, // magic
        0x00, 0x00, // minor version
        0x00, 0x34, // major version (Java 8)
        0x00, 0x03, // constant pool count
        0x01, 0x00, 0x01, b'A', // Utf8 "A"
        0x07, 0x00, 0x01, // Class #1
        0x00, 0x21, // access flags: public | super
        0x00, 0x02, // this class
        0x00, 0x00, // super class
        0x00, 0x00, // interfaces
        0x00, 0x00, // fields
        0x00, 0x00, // methods
        0x00, 0x00, // attributes
    ];

    const BROKEN_CLASS: &[u8] = b"not a class file";

    fn jar(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

        for (name, contents) in entries {
            zip.start_file(*name, SimpleFileOptions::default()).unwrap();
            zip.write_all(contents).unwrap();
        }

        zip.finish().unwrap().into_inner()
    }

    fn mixed_jar() -> Vec<u8> {
        jar(&[
            ("com/example/Valid.class", VALID_CLASS),
            ("com/example/Broken.class", BROKEN_CLASS),
        ])
    }

    #[test]
    fn scan_reports_classes_that_could_not_be_parsed() {
        let scan = scan_classes_reader(&mut Cursor::new(mixed_jar())).unwrap();

        assert_eq!(scan.items.len(), 1);
        assert_eq!(scan.items[0].name, "com/example/Valid.class");

        assert_eq!(scan.skipped.len(), 1);
        assert_eq!(scan.skipped[0].name, "com/example/Broken.class");
        assert_ne!(scan.skipped[0].reason, "");
    }

    #[test]
    fn list_classes_still_returns_only_parsed_classes() {
        let entries = list_classes_reader(&mut Cursor::new(mixed_jar())).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "com/example/Valid.class");
    }

    #[test]
    fn classes_in_embedded_jars_are_labeled_with_their_jar() {
        let nested = jar(&[("com/example/Inner.class", VALID_CLASS)]);
        let outer = jar(&[
            ("com/example/Outer.class", VALID_CLASS),
            ("META-INF/jarjar/inner.jar", &nested),
        ]);

        let entries = list_classes_reader(&mut Cursor::new(outer)).unwrap();
        let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"com/example/Outer.class"));
        assert!(
            names.contains(&"META-INF/jarjar/inner.jar!/com/example/Inner.class"),
            "nested class keeps its jar as a prefix, got {names:?}"
        );
    }

    #[test]
    fn grep_and_mixins_report_the_same_skips() {
        let mut cursor = Cursor::new(mixed_jar());
        let grep = scan_grep_reader(&mut cursor, "java/lang/Object").unwrap();
        assert_eq!(grep.skipped.len(), 1);

        let mut cursor = Cursor::new(mixed_jar());
        let mixins = scan_mixins_reader(&mut cursor).unwrap();
        assert_eq!(mixins.skipped.len(), 1);
    }

    #[test]
    fn group_mixin_overlaps_returns_targets_in_multiple_jars() {
        let entries = vec![
            (
                "a.jar".to_owned(),
                MixinEntry {
                    mixin_class: "MixA".to_owned(),
                    target: "net/minecraft/Foo".to_owned(),
                },
            ),
            (
                "b.jar".to_owned(),
                MixinEntry {
                    mixin_class: "MixB".to_owned(),
                    target: "net/minecraft/Foo".to_owned(),
                },
            ),
            (
                "a.jar".to_owned(),
                MixinEntry {
                    mixin_class: "MixC".to_owned(),
                    target: "net/minecraft/Bar".to_owned(),
                },
            ),
        ];

        let overlaps = group_mixin_overlaps(&entries);

        assert_eq!(overlaps.len(), 1, "only Foo is targeted by multiple JARs");
        assert_eq!(overlaps[0].target, "net/minecraft/Foo");
        assert_eq!(overlaps[0].mixins.len(), 2);
        assert_eq!(overlaps[0].mixins[0].jar, "a.jar");
        assert_eq!(overlaps[0].mixins[0].mixin_class, "MixA");
        assert_eq!(overlaps[0].mixins[1].jar, "b.jar");
        assert_eq!(overlaps[0].mixins[1].mixin_class, "MixB");
    }

    #[test]
    fn group_mixin_overlaps_ignores_single_jar_targets() {
        let entries = vec![
            (
                "a.jar".to_owned(),
                MixinEntry {
                    mixin_class: "MixA".to_owned(),
                    target: "net/minecraft/Foo".to_owned(),
                },
            ),
            (
                "a.jar".to_owned(),
                MixinEntry {
                    mixin_class: "MixB".to_owned(),
                    target: "net/minecraft/Foo".to_owned(),
                },
            ),
        ];

        // Both mixins are from the same JAR, so no overlap.
        assert!(group_mixin_overlaps(&entries).is_empty());
    }

    #[test]
    fn find_mixin_overlaps_buffers_empty() {
        assert!(find_mixin_overlaps_buffers(&[]).unwrap().is_empty());
    }
}
