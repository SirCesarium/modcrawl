//! Class-file inspection: class listings, constant-pool search, mixin targets,
//! and duplicate-class detection.

use std::any::Any;
use std::io::{Read, Seek};
use std::panic;
use std::path::Path;

use java_class_rs::{
    ClassAccessFlags, ClassFile, ConstantPoolEntry, ElementValue, ParsedAttribute, get_entry,
    get_utf8, parse_classfile, parse_specialized_attribute,
};
use serde::Serialize;
use zipcrawl::ZipManager;

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
    scan_jar(manager, |cf, name| {
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
    scan_jar(manager, |cf, name| search_constant_pool(cf, name, pattern))
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

fn mixins_impl<R: Read + Seek>(manager: &mut ZipManager<R>) -> Result<ClassScan<MixinEntry>> {
    scan_jar(manager, extract_mixin_targets)
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

/// Scan every `.class` entry in `manager`, collecting what `collect` returns
/// for each parsed class and recording every class that was skipped.
fn scan_jar<R, T, F>(manager: &mut ZipManager<R>, mut collect: F) -> Result<ClassScan<T>>
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

        match parse_class_in_jar(manager, &entry.name)? {
            ClassParse::Parsed(cf) => items.extend(collect(&cf, &entry.name)),
            ClassParse::Skipped(reason) => skipped.push(SkippedClass {
                name: entry.name.clone(),
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
    fn grep_and_mixins_report_the_same_skips() {
        let mut cursor = Cursor::new(mixed_jar());
        let grep = scan_grep_reader(&mut cursor, "java/lang/Object").unwrap();
        assert_eq!(grep.skipped.len(), 1);

        let mut cursor = Cursor::new(mixed_jar());
        let mixins = scan_mixins_reader(&mut cursor).unwrap();
        assert_eq!(mixins.skipped.len(), 1);
    }
}
