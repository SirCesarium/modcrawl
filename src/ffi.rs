use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;
use std::slice;

use crate::core::dep;
use crate::core::identify::{identify, identify_reader};
use crate::core::metadata;
use crate::error::Result;

#[cfg(feature = "classfile")]
use crate::core::classfile;

#[cfg(feature = "classfile")]
use std::path::PathBuf;

fn to_c_string(result: Result<String>) -> *mut c_char {
    match result {
        Ok(s) => match CString::new(s) {
            Ok(cs) => cs.into_raw(),
            Err(e) => {
                eprintln!("modcrawl: CString error: {e}");
                ptr::null_mut()
            }
        },
        Err(e) => {
            eprintln!("modcrawl: {e:?}");
            ptr::null_mut()
        }
    }
}

/// Detect the mod/plugin type from a JAR file path.
///
/// Returns a C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_identify(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let Ok(path_str) = unsafe { CStr::from_ptr(path) }.to_str() else {
        return ptr::null_mut();
    };
    to_c_string(identify(Path::new(path_str)).map(|m| m.to_string()))
}

/// Detect the mod/plugin type from a ZIP/JAR byte buffer.
///
/// Returns a C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `data` must be valid for reads of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_identify_bytes(data: *const u8, len: usize) -> *mut c_char {
    if data.is_null() || len == 0 {
        return ptr::null_mut();
    }
    let slice = unsafe { slice::from_raw_parts(data, len) };
    let mut cursor = Cursor::new(slice);
    to_c_string(identify_reader(&mut cursor).map(|m| m.to_string()))
}

/// Read and parse metadata from a JAR file path.
///
/// Returns a human-readable C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_metadata(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let Ok(path_str) = unsafe { CStr::from_ptr(path) }.to_str() else {
        return ptr::null_mut();
    };
    to_c_string(metadata::read_metadata(Path::new(path_str)).map(|m| m.to_string()))
}

/// Read and parse metadata from a JAR file path, returning JSON.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_metadata_json(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let Ok(path_str) = unsafe { CStr::from_ptr(path) }.to_str() else {
        return ptr::null_mut();
    };
    to_c_string(
        metadata::read_metadata(Path::new(path_str))
            .and_then(|m| serde_json::to_string(&m).map_err(Into::into)),
    )
}

/// Read and parse metadata from a ZIP/JAR byte buffer.
///
/// Returns a human-readable C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `data` must be valid for reads of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_metadata_bytes(data: *const u8, len: usize) -> *mut c_char {
    if data.is_null() || len == 0 {
        return ptr::null_mut();
    }
    let slice = unsafe { slice::from_raw_parts(data, len) };
    let mut cursor = Cursor::new(slice);
    to_c_string(metadata::read_metadata_reader(&mut cursor).map(|m| m.to_string()))
}

/// Read and parse metadata from a ZIP/JAR byte buffer, returning JSON.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `data` must be valid for reads of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_metadata_json_bytes(data: *const u8, len: usize) -> *mut c_char {
    if data.is_null() || len == 0 {
        return ptr::null_mut();
    }
    let slice = unsafe { slice::from_raw_parts(data, len) };
    let mut cursor = Cursor::new(slice);
    to_c_string(
        metadata::read_metadata_reader(&mut cursor)
            .and_then(|m| serde_json::to_string(&m).map_err(Into::into)),
    )
}

/// Analyze dependencies from a JAR file path.
///
/// `include_jij` controls whether embedded JAR-in-JAR entries appear in the output.
///
/// Returns a human-readable C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_deps(path: *const c_char, include_jij: bool) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let Ok(path_str) = unsafe { CStr::from_ptr(path) }.to_str() else {
        return ptr::null_mut();
    };
    to_c_string(dep::analyze(Path::new(path_str), include_jij).map(|r| r.to_string()))
}

/// Analyze dependencies from a JAR file path, returning JSON.
///
/// `include_jij` controls whether embedded JAR-in-JAR entries appear in the output.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_deps_json(path: *const c_char, include_jij: bool) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let Ok(path_str) = unsafe { CStr::from_ptr(path) }.to_str() else {
        return ptr::null_mut();
    };
    to_c_string(
        dep::analyze(Path::new(path_str), include_jij)
            .and_then(|r| serde_json::to_string(&r).map_err(Into::into)),
    )
}

/// Analyze dependencies from a ZIP/JAR byte buffer.
///
/// `include_jij` controls whether embedded JAR-in-JAR entries appear in the output.
///
/// Returns a human-readable C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `data` must be valid for reads of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_deps_bytes(
    data: *const u8,
    len: usize,
    include_jij: bool,
) -> *mut c_char {
    if data.is_null() || len == 0 {
        return ptr::null_mut();
    }
    let slice = unsafe { slice::from_raw_parts(data, len) };
    let mut cursor = Cursor::new(slice);
    to_c_string(dep::analyze_reader(&mut cursor, include_jij).map(|r| r.to_string()))
}

/// Analyze dependencies from a ZIP/JAR byte buffer, returning JSON.
///
/// `include_jij` controls whether embedded JAR-in-JAR entries appear in the output.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `data` must be valid for reads of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_deps_json_bytes(
    data: *const u8,
    len: usize,
    include_jij: bool,
) -> *mut c_char {
    if data.is_null() || len == 0 {
        return ptr::null_mut();
    }
    let slice = unsafe { slice::from_raw_parts(data, len) };
    let mut cursor = Cursor::new(slice);
    to_c_string(
        dep::analyze_reader(&mut cursor, include_jij)
            .and_then(|r| serde_json::to_string(&r).map_err(Into::into)),
    )
}

/// List all .class files from a JAR, returning JSON.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[cfg(feature = "classfile")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_classes_json(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let Ok(path_str) = unsafe { CStr::from_ptr(path) }.to_str() else {
        return ptr::null_mut();
    };
    to_c_string(
        classfile::list_classes(Path::new(path_str))
            .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
    )
}

/// Search the constant pool of all .class files in a JAR, returning JSON.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
/// `pattern` must be a valid null-terminated C string.
#[cfg(feature = "classfile")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_grep_json(
    path: *const c_char,
    pattern: *const c_char,
) -> *mut c_char {
    if path.is_null() || pattern.is_null() {
        return ptr::null_mut();
    }
    let Ok(path_str) = unsafe { CStr::from_ptr(path) }.to_str() else {
        return ptr::null_mut();
    };
    let Ok(pattern_str) = unsafe { CStr::from_ptr(pattern) }.to_str() else {
        return ptr::null_mut();
    };
    to_c_string(
        classfile::grep(Path::new(path_str), pattern_str)
            .and_then(|matches| serde_json::to_string(&matches).map_err(Into::into)),
    )
}

/// Extract mixin targets from a JAR, returning JSON.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[cfg(feature = "classfile")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_mixins_json(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let Ok(path_str) = unsafe { CStr::from_ptr(path) }.to_str() else {
        return ptr::null_mut();
    };
    to_c_string(
        classfile::mixins(Path::new(path_str))
            .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
    )
}

/// Find duplicate class entries across multiple JARs, returning JSON.
///
/// `paths` is a null-terminated array of null-terminated strings.
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `paths` must be a valid null-terminated array of valid null-terminated C strings.
#[cfg(feature = "classfile")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_dupes_json(paths: *mut *const c_char) -> *mut c_char {
    if paths.is_null() {
        return ptr::null_mut();
    }
    let mut path_vec: Vec<PathBuf> = Vec::new();
    let mut i = 0;
    loop {
        let p = unsafe { *paths.add(i) };
        if p.is_null() {
            break;
        }
        if let Ok(s) = unsafe { CStr::from_ptr(p) }.to_str() {
            path_vec.push(PathBuf::from(s));
        }
        i += 1;
    }
    let path_refs: Vec<&Path> = path_vec.iter().map(PathBuf::as_path).collect();
    to_c_string(
        classfile::find_duplicates(&path_refs)
            .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
    )
}

/// Free a string returned by any `modcrawl_*` function.
///
/// # Safety
///
/// `s` must be a pointer previously returned by a `modcrawl_*` function,
/// or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
