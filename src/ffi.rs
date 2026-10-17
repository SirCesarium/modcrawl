//! C bindings for the core API.
//!
//! # Contract
//!
//! - Every function returning `char *` hands ownership of a heap-allocated,
//!   NUL-terminated string to the caller. Free it with [`modcrawl_free_string`],
//!   never with `free()`: the allocation belongs to Rust's allocator.
//! - NULL means an error, and the reason is printed to stderr.
//! - Interior NUL characters found in mod data are replaced with U+FFFD, so a
//!   crafted JAR can never turn a successful parse into an error return.
//! - Every function is safe to call concurrently from multiple threads.
//! - Panics are contained: they are reported on stderr and surface as NULL
//!   instead of aborting the host process.

use std::any::Any;
use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
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

/// Run `f` behind a panic firewall.
///
/// A panic unwinding out of an `extern "C"` function aborts the entire host
/// process, so every export funnels through here: the panic is reported on
/// stderr and the caller receives NULL rather than a dead process.
///
/// The message is re-printed from the payload on purpose. The default panic
/// hook already printed the full report, but a host that installed its own
/// hook — a Rust program linking this as a staticlib, say — may have swallowed
/// it, and "NULL with no explanation" is exactly the failure this firewall
/// exists to avoid.
fn guard(f: impl FnOnce() -> *mut c_char) -> *mut c_char {
    let outcome = catch_unwind(AssertUnwindSafe(f));
    if let Err(panic) = &outcome {
        // `&**panic`, deliberately not `panic`: `&Box<dyn Any + Send>` coerces to
        // the *Box's own* `Any` impl, so the payload type seen by the callee
        // would be `Box<dyn Any + Send>` and nothing would ever downcast.
        eprintln!(
            "modcrawl: internal panic{}; returning NULL",
            panic_message(&**panic)
        );
        return ptr::null_mut();
    }
    // Reached only when the closure succeeded: the branch above is the sole
    // other exit, and it returns early.
    outcome.unwrap_or(ptr::null_mut())
}

/// Best-effort description of a panic payload, prefixed with `: `.
///
/// Callers must pass the *inner* trait object (`&**box`, or `&*box` from an
/// owned `Box`); see `guard` for why the coercion is a trap.
///
/// The payload type is whatever `panic!` was given, so anything else — a panic
/// raised through `resume_unwind`, say — yields an empty string.
fn panic_message(panic: &(dyn Any + Send)) -> String {
    if let Some(text) = panic.downcast_ref::<&str>() {
        format!(": {text}")
    } else if let Some(text) = panic.downcast_ref::<String>() {
        format!(": {text}")
    } else {
        String::new()
    }
}

/// Drop the interior NULs that a C string cannot represent.
///
/// Without this, a mod whose metadata contains a NUL (a crafted
/// `fabric.mod.json`, say) would make the call fail with "nul byte found in
/// provided data" even though parsing succeeded.
fn sanitize(s: String) -> String {
    if s.contains('\0') {
        s.replace('\0', "\u{FFFD}")
    } else {
        s
    }
}

/// Turn a parsed result into an owned C string, or NULL on error.
///
/// `sanitize` has already stripped every interior NUL, so `CString::new`
/// cannot fail here; an empty string is returned rather than panicking across
/// the boundary on a path that cannot happen.
fn to_c_string(result: Result<String>) -> *mut c_char {
    match result {
        Ok(s) => CString::new(sanitize(s)).unwrap_or_default().into_raw(),
        Err(e) => {
            eprintln!("modcrawl: {e}");
            ptr::null_mut()
        }
    }
}

/// Borrow a NUL-terminated C string as UTF-8, or NULL for invalid input.
///
/// # Safety
///
/// `s` must be NULL, or a valid NUL-terminated C string that stays valid for
/// the duration of the call.
unsafe fn cstr<'a>(s: *const c_char) -> Option<&'a str> {
    if s.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(s) }.to_str().ok()
}

/// Borrow `len` bytes from `data`, or NULL for invalid input.
///
/// # Safety
///
/// `data` must be NULL, or valid for reads of `len` bytes for the duration of
/// the call.
unsafe fn bytes<'a>(data: *const u8, len: usize) -> Option<&'a [u8]> {
    if data.is_null() || len == 0 {
        return None;
    }
    Some(unsafe { slice::from_raw_parts(data, len) })
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
    guard(|| {
        let Some(path_str) = (unsafe { cstr(path) }) else {
            return ptr::null_mut();
        };
        to_c_string(identify(Path::new(path_str)).map(|m| m.to_string()))
    })
}

/// Detect the mod/plugin type from a ZIP/JAR byte buffer.
///
/// Returns a C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error or when `len` is 0.
///
/// # Safety
///
/// `data` must be valid for reads of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_identify_bytes(data: *const u8, len: usize) -> *mut c_char {
    guard(|| {
        let Some(buf) = (unsafe { bytes(data, len) }) else {
            return ptr::null_mut();
        };
        let mut cursor = Cursor::new(buf);
        to_c_string(identify_reader(&mut cursor).map(|m| m.to_string()))
    })
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
    guard(|| {
        let Some(path_str) = (unsafe { cstr(path) }) else {
            return ptr::null_mut();
        };
        to_c_string(metadata::read_metadata(Path::new(path_str)).map(|m| m.to_string()))
    })
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
    guard(|| {
        let Some(path_str) = (unsafe { cstr(path) }) else {
            return ptr::null_mut();
        };
        to_c_string(
            metadata::read_metadata(Path::new(path_str))
                .and_then(|m| serde_json::to_string(&m).map_err(Into::into)),
        )
    })
}

/// Read and parse metadata from a ZIP/JAR byte buffer.
///
/// Returns a human-readable C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error or when `len` is 0.
///
/// # Safety
///
/// `data` must be valid for reads of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_metadata_bytes(data: *const u8, len: usize) -> *mut c_char {
    guard(|| {
        let Some(buf) = (unsafe { bytes(data, len) }) else {
            return ptr::null_mut();
        };
        let mut cursor = Cursor::new(buf);
        to_c_string(metadata::read_metadata_reader(&mut cursor).map(|m| m.to_string()))
    })
}

/// Read and parse metadata from a ZIP/JAR byte buffer, returning JSON.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error or when `len` is 0.
///
/// # Safety
///
/// `data` must be valid for reads of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_metadata_json_bytes(data: *const u8, len: usize) -> *mut c_char {
    guard(|| {
        let Some(buf) = (unsafe { bytes(data, len) }) else {
            return ptr::null_mut();
        };
        let mut cursor = Cursor::new(buf);
        to_c_string(
            metadata::read_metadata_reader(&mut cursor)
                .and_then(|m| serde_json::to_string(&m).map_err(Into::into)),
        )
    })
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
    guard(|| {
        let Some(path_str) = (unsafe { cstr(path) }) else {
            return ptr::null_mut();
        };
        to_c_string(dep::analyze(Path::new(path_str), include_jij).map(|r| r.to_string()))
    })
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
    guard(|| {
        let Some(path_str) = (unsafe { cstr(path) }) else {
            return ptr::null_mut();
        };
        to_c_string(
            dep::analyze(Path::new(path_str), include_jij)
                .and_then(|r| serde_json::to_string(&r).map_err(Into::into)),
        )
    })
}

/// Analyze dependencies from a ZIP/JAR byte buffer.
///
/// `include_jij` controls whether embedded JAR-in-JAR entries appear in the output.
///
/// Returns a human-readable C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error or when `len` is 0.
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
    guard(|| {
        let Some(buf) = (unsafe { bytes(data, len) }) else {
            return ptr::null_mut();
        };
        let mut cursor = Cursor::new(buf);
        to_c_string(dep::analyze_reader(&mut cursor, include_jij).map(|r| r.to_string()))
    })
}

/// Analyze dependencies from a ZIP/JAR byte buffer, returning JSON.
///
/// `include_jij` controls whether embedded JAR-in-JAR entries appear in the output.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error or when `len` is 0.
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
    guard(|| {
        let Some(buf) = (unsafe { bytes(data, len) }) else {
            return ptr::null_mut();
        };
        let mut cursor = Cursor::new(buf);
        to_c_string(
            dep::analyze_reader(&mut cursor, include_jij)
                .and_then(|r| serde_json::to_string(&r).map_err(Into::into)),
        )
    })
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
    guard(|| {
        let Some(path_str) = (unsafe { cstr(path) }) else {
            return ptr::null_mut();
        };
        to_c_string(
            classfile::list_classes(Path::new(path_str))
                .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
        )
    })
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
    guard(|| {
        let (Some(path_str), Some(pattern_str)) = (unsafe { cstr(path) }, unsafe { cstr(pattern) })
        else {
            return ptr::null_mut();
        };
        to_c_string(
            classfile::grep(Path::new(path_str), pattern_str)
                .and_then(|matches| serde_json::to_string(&matches).map_err(Into::into)),
        )
    })
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
    guard(|| {
        let Some(path_str) = (unsafe { cstr(path) }) else {
            return ptr::null_mut();
        };
        to_c_string(
            classfile::mixins(Path::new(path_str))
                .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
        )
    })
}

/// Find duplicate class entries across multiple JARs, returning JSON.
///
/// `paths` is a NULL-terminated array of NULL-terminated strings, declared as
/// `char *const *` so that the canonical C call passes `char *paths[]`.
/// Every path must be valid UTF-8; one that is not is an error, not a silently
/// skipped JAR. An empty list is valid and returns `[]`.
///
/// Returns a JSON C string that must be freed with `modcrawl_free_string`.
/// Returns NULL on error.
///
/// # Safety
///
/// `paths` must be a valid NULL-terminated array of valid NULL-terminated C
/// strings that stays valid for the duration of the call.
#[cfg(feature = "classfile")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modcrawl_dupes_json(paths: *const *mut c_char) -> *mut c_char {
    guard(|| {
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
            let Ok(s) = unsafe { CStr::from_ptr(p) }.to_str() else {
                eprintln!("modcrawl: dupes: path #{i} is not valid UTF-8");
                return ptr::null_mut();
            };
            path_vec.push(PathBuf::from(s));
            i += 1;
        }
        let path_refs: Vec<&Path> = path_vec.iter().map(PathBuf::as_path).collect();
        to_c_string(
            classfile::find_duplicates(&path_refs)
                .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
        )
    })
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::any::Any;
    use std::env;
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::thread;

    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    use super::*;

    /// A minimal but complete Fabric mod descriptor.
    const FABRIC_MOD_JSON: &str = r#"{"schemaVersion":1,"id":"testmod","name":"Test Mod","version":"1.0.0","environment":"*","depends":{"fabricloader":">=0.15.0"}}"#;

    /// A valid class file header plus padding: `find_duplicates` only looks at
    /// entry names, and the classfile parsers reject the body without panicking.
    #[cfg(feature = "classfile")]
    const FAKE_CLASS: &[u8] = b"\xca\xfe\xba\xbe\x00\x00\x00\x34 padding";

    /// A scratch directory that removes itself when the test ends, so the suite
    /// does not pile JARs up in the system temp directory.
    struct TestDir(PathBuf);

    impl TestDir {
        fn new(test: &str) -> Self {
            let dir = env::temp_dir().join(format!("modcrawl-ffi-{}-{test}", process::id()));
            fs::create_dir_all(&dir).unwrap();
            TestDir(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Write a JAR holding the given `(entry name, contents)` pairs.
    fn write_jar(dir: &Path, name: &str, entries: &[(&str, &[u8])]) -> PathBuf {
        let path = dir.join(name);
        let file = fs::File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        for (entry, contents) in entries {
            zip.start_file(*entry, SimpleFileOptions::default()).unwrap();
            zip.write_all(contents).unwrap();
        }
        zip.finish().unwrap();
        path
    }

    fn fabric_jar(dir: &Path, name: &str) -> PathBuf {
        write_jar(dir, name, &[("fabric.mod.json", FABRIC_MOD_JSON.as_bytes())])
    }

    /// Take a returned string the way a C caller would: read it, then free it.
    unsafe fn take(s: *mut c_char) -> Option<String> {
        if s.is_null() {
            return None;
        }
        let out = unsafe { CStr::from_ptr(s) }.to_str().unwrap().to_owned();
        unsafe { modcrawl_free_string(s) };
        Some(out)
    }

    fn c_path(path: &Path) -> CString {
        CString::new(path.to_str().unwrap()).unwrap()
    }

    #[test]
    fn identify_from_path_and_bytes() {
        let dir = TestDir::new("identify");
        let jar = fabric_jar(dir.path(), "identify.jar");
        let path = c_path(&jar);

        let from_path = unsafe { take(modcrawl_identify(path.as_ptr())) };
        assert_eq!(from_path.as_deref(), Some("Fabric"));

        let data = fs::read(&jar).unwrap();
        let from_bytes =
            unsafe { take(modcrawl_identify_bytes(data.as_ptr(), data.len())) };
        assert_eq!(from_bytes.as_deref(), Some("Fabric"));
    }

    #[test]
    fn identify_rejects_null_empty_and_invalid_input() {
        let dir = TestDir::new("identify-rejects");
        assert!(unsafe { take(modcrawl_identify(ptr::null())) }.is_none());
        assert!(unsafe { take(modcrawl_identify_bytes(ptr::null(), 16)) }.is_none());
        assert!(unsafe { take(modcrawl_identify_bytes(b"PK\x03\x04".as_ptr(), 0)) }.is_none());

        let not_a_jar = dir.path().join("not-a-jar.bin");
        fs::write(&not_a_jar, b"this is not a zip archive").unwrap();
        let path = c_path(&not_a_jar);
        assert!(unsafe { take(modcrawl_identify(path.as_ptr())) }.is_none());

        let missing = c_path(&dir.path().join("does-not-exist.jar"));
        assert!(unsafe { take(modcrawl_identify(missing.as_ptr())) }.is_none());
    }

    #[test]
    fn metadata_from_path_and_bytes() {
        let dir = TestDir::new("metadata");
        let jar = fabric_jar(dir.path(), "metadata.jar");
        let path = c_path(&jar);

        let human = unsafe { take(modcrawl_metadata(path.as_ptr())) }.unwrap();
        assert!(human.contains("Test Mod"), "unexpected output: {human}");
        assert!(human.contains("testmod"), "unexpected output: {human}");

        let json = unsafe { take(modcrawl_metadata_json(path.as_ptr())) }.unwrap();
        assert!(json.contains("\"testmod\""), "unexpected output: {json}");

        let data = fs::read(&jar).unwrap();
        let bytes_json = unsafe {
            take(modcrawl_metadata_json_bytes(data.as_ptr(), data.len()))
        }
        .unwrap();
        assert_eq!(bytes_json, json);

        let bytes_human =
            unsafe { take(modcrawl_metadata_bytes(data.as_ptr(), data.len())) }.unwrap();
        assert_eq!(bytes_human, human);
    }

    /// Regression test: an interior NUL in mod data used to fail the whole call
    /// with "nul byte found in provided data" even though parsing succeeded.
    #[test]
    fn metadata_with_interior_nul_still_succeeds() {
        // `\u0000` in the JSON *text* is legal; once parsed it is a real NUL.
        let nul_escape = format!("\\u{:04x}", 0);
        let with_nul = format!(
            r#"{{"schemaVersion":1,"id":"nulmod","name":"bad{nul_escape}name","version":"1.0"}}"#
        );
        let dir = TestDir::new("nul");
        let jar = write_jar(dir.path(), "nul.jar", &[("fabric.mod.json", with_nul.as_bytes())]);
        let path = c_path(&jar);

        let human = unsafe { take(modcrawl_metadata(path.as_ptr())) }.unwrap();
        assert!(!human.contains('\0'), "NUL leaked into the C string: {human:?}");
        assert!(human.contains("bad\u{FFFD}name"), "unexpected output: {human}");

        // The JSON variant escapes the NUL, so it round-trips intact.
        let json = unsafe { take(modcrawl_metadata_json(path.as_ptr())) }.unwrap();
        assert!(json.contains(r"bad\u0000name"), "unexpected output: {json}");
    }

    #[test]
    fn deps_from_path_and_bytes() {
        let dir = TestDir::new("deps");
        let jar = fabric_jar(dir.path(), "deps.jar");
        let path = c_path(&jar);

        let human = unsafe { take(modcrawl_deps(path.as_ptr(), false)) }.unwrap();
        assert!(human.contains("fabricloader"), "unexpected output: {human}");

        let json = unsafe { take(modcrawl_deps_json(path.as_ptr(), true)) }.unwrap();
        assert!(json.contains("fabricloader"), "unexpected output: {json}");
        assert!(json.contains("Required"), "unexpected output: {json}");

        let data = fs::read(&jar).unwrap();
        let bytes_json = unsafe {
            take(modcrawl_deps_json_bytes(data.as_ptr(), data.len(), false))
        }
        .unwrap();
        assert!(bytes_json.contains("fabricloader"), "unexpected: {bytes_json}");

        assert!(unsafe { take(modcrawl_deps(ptr::null(), false)) }.is_none());
        assert!(unsafe { take(modcrawl_deps_json_bytes(data.as_ptr(), 0, false)) }.is_none());
    }

    #[cfg(feature = "classfile")]
    #[test]
    fn classes_grep_and_mixins_on_a_classless_jar() {
        let dir = TestDir::new("classless");
        let jar = fabric_jar(dir.path(), "classless.jar");
        let path = c_path(&jar);

        assert_eq!(
            unsafe { take(modcrawl_classes_json(path.as_ptr())) }.as_deref(),
            Some("[]")
        );
        assert_eq!(
            unsafe { take(modcrawl_mixins_json(path.as_ptr())) }.as_deref(),
            Some("[]")
        );
        let pattern = CString::new("fabric").unwrap();
        assert_eq!(
            unsafe { take(modcrawl_grep_json(path.as_ptr(), pattern.as_ptr())) }.as_deref(),
            Some("[]")
        );

        assert!(unsafe { take(modcrawl_classes_json(ptr::null())) }.is_none());
        assert!(unsafe { take(modcrawl_grep_json(path.as_ptr(), ptr::null())) }.is_none());
    }

    #[cfg(feature = "classfile")]
    #[test]
    fn dupes_reports_classes_shared_between_jars() {
        let dir = TestDir::new("dupes-shared");
        let a = write_jar(
            dir.path(),
            "dupes-a.jar",
            &[
                ("fabric.mod.json", FABRIC_MOD_JSON.as_bytes()),
                ("com/example/Shared.class", FAKE_CLASS),
            ],
        );
        let b = write_jar(
            dir.path(),
            "dupes-b.jar",
            &[
                ("fabric.mod.json", FABRIC_MOD_JSON.as_bytes()),
                ("com/example/Shared.class", FAKE_CLASS),
                ("com/example/OnlyInB.class", FAKE_CLASS),
            ],
        );
        let a = CString::new(a.to_str().unwrap()).unwrap();
        let b = CString::new(b.to_str().unwrap()).unwrap();

        let paths: [*mut c_char; 3] = [
            a.as_ptr().cast_mut(),
            b.as_ptr().cast_mut(),
            ptr::null_mut(),
        ];
        let json = unsafe { take(modcrawl_dupes_json(paths.as_ptr())) }.unwrap();
        assert!(json.contains("com/example/Shared.class"), "unexpected: {json}");
        assert!(json.contains("dupes-a.jar"), "unexpected: {json}");
        assert!(json.contains("dupes-b.jar"), "unexpected: {json}");
        assert!(!json.contains("OnlyInB"), "unexpected: {json}");
    }

    #[cfg(feature = "classfile")]
    #[test]
    fn dupes_rejects_null_and_non_utf8_input() {
        let dir = TestDir::new("dupes-rejects");
        assert!(unsafe { take(modcrawl_dupes_json(ptr::null())) }.is_none());

        // A NULL-terminated empty list is valid: no jars, no dupes.
        let empty: [*mut c_char; 1] = [ptr::null_mut()];
        assert_eq!(
            unsafe { take(modcrawl_dupes_json(empty.as_ptr())) }.as_deref(),
            Some("[]")
        );

        // A path that is not valid UTF-8 is an error, not a silently skipped jar.
        let good = CString::new(fabric_jar(dir.path(), "dupes-good.jar").to_str().unwrap()).unwrap();
        let bad = CString::new(b"/tmp/not-utf8-\xff.jar".to_vec()).unwrap();
        let paths: [*mut c_char; 3] = [
            good.as_ptr().cast_mut(),
            bad.as_ptr().cast_mut(),
            ptr::null_mut(),
        ];
        assert!(unsafe { take(modcrawl_dupes_json(paths.as_ptr())) }.is_none());
    }

    #[test]
    fn free_string_takes_null() {
        unsafe { modcrawl_free_string(ptr::null_mut()) };
    }

    /// A panic must reach the caller as NULL instead of aborting the process,
    /// and its message must survive a host that replaced the panic hook.
    #[test]
    fn panics_are_contained() {
        let result = guard(|| panic!("boom behind the FFI boundary"));
        assert!(result.is_null());

        let borrowed: Box<dyn Any + Send> = Box::new("boom");
        assert_eq!(panic_message(&*borrowed), ": boom");

        let owned: Box<dyn Any + Send> = Box::new(String::from("boom"));
        assert_eq!(panic_message(&*owned), ": boom");

        // `resume_unwind` and friends carry payloads we cannot describe.
        let other: Box<dyn Any + Send> = Box::new(42_u8);
        assert_eq!(panic_message(&*other), "");
    }

    /// The registry is a shared global, so the exports must be usable in parallel.
    #[test]
    fn concurrent_calls_are_safe() {
        let dir = TestDir::new("concurrent");
        let jar = fabric_jar(dir.path(), "concurrent.jar");
        let path = c_path(&jar);

        thread::scope(|scope| {
            for _ in 0..8 {
                let path = &path;
                scope.spawn(move || {
                    for _ in 0..20 {
                        let out = unsafe { take(modcrawl_identify(path.as_ptr())) };
                        assert_eq!(out.as_deref(), Some("Fabric"));
                    }
                });
            }
        });
    }
}
