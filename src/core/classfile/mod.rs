use std::io::Read;
use std::path::Path;

use java_class_rs::{
    get_entry, get_utf8, parse_classfile, ClassAccessFlags, ClassFile, ConstantPoolEntry,
};
use serde::Serialize;
use zipcrawl::ZipManager;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize)]
pub struct ClassEntry {
    pub name: String,
    pub java_version: String,
    pub access_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GrepMatch {
    pub class_file: String,
    pub pool_tag: String,
    pub value: String,
}

/// List all `.class` files in a JAR with basic metadata.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn list_classes(path: &Path) -> Result<Vec<ClassEntry>> {
    let mut manager = ZipManager::new(path)?;
    let entries = manager.entries()?;
    let mut classes = Vec::new();

    for entry in &entries {
        if !is_class_entry(&entry.name) || entry.is_dir {
            continue;
        }

        let Ok(Some(cf)) = parse_class_in_jar(&mut manager, &entry.name) else {
            continue;
        };

        let version = java_version_string(cf.major_version, cf.minor_version);
        let flags = format_class_access_flags(cf.access_flags);

        classes.push(ClassEntry {
            name: entry.name.clone(),
            java_version: version,
            access_flags: flags,
        });
    }

    Ok(classes)
}

/// Search the constant pool of all `.class` files in a JAR for a pattern.
///
/// # Errors
///
/// Returns an error if the JAR cannot be read.
pub fn grep(path: &Path, pattern: &str) -> Result<Vec<GrepMatch>> {
    let mut manager = ZipManager::new(path)?;
    let entries = manager.entries()?;
    let mut matches = Vec::new();

    for entry in &entries {
        if !is_class_entry(&entry.name) || entry.is_dir {
            continue;
        }

        let Ok(Some(cf)) = parse_class_in_jar(&mut manager, &entry.name) else {
            continue;
        };

        let class_matches = search_constant_pool(&cf, &entry.name, pattern);
        matches.extend(class_matches);
    }

    Ok(matches)
}

fn is_class_entry(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("class"))
}

fn parse_class_in_jar(
    manager: &mut ZipManager,
    name: &str,
) -> Result<Option<ClassFile>> {
    let mut file = manager.open_file(name)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(Error::Io)?;

    match parse_classfile(&bytes) {
        Ok((_, cf)) => Ok(Some(cf)),
        Err(_) => Ok(None),
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
                if let Some(resolved) =
                    resolve_ref(&cf.constant_pool, info.class_index, info.name_and_type_index)
                    && resolved.contains(pattern)
                {
                    results.push(GrepMatch {
                        class_file: class_name.to_string(),
                        pool_tag: "MethodRef".to_string(),
                        value: resolved,
                    });
                }
            }
            ConstantPoolEntry::Fieldref(info) => {
                if let Some(resolved) =
                    resolve_ref(&cf.constant_pool, info.class_index, info.name_and_type_index)
                    && resolved.contains(pattern)
                {
                    results.push(GrepMatch {
                        class_file: class_name.to_string(),
                        pool_tag: "FieldRef".to_string(),
                        value: resolved,
                    });
                }
            }
            ConstantPoolEntry::InterfaceMethodref(info) => {
                if let Some(resolved) =
                    resolve_ref(&cf.constant_pool, info.class_index, info.name_and_type_index)
                    && resolved.contains(pattern)
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

fn resolve_ref(
    cp: &[ConstantPoolEntry],
    class_index: u16,
    nat_index: u16,
) -> Option<String> {
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
