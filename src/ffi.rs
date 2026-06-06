use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;
use std::slice;

use crate::core::identify::{identify, identify_reader};
use crate::error::Result;

fn to_c_string(result: Result<String>) -> *mut c_char {
    match result {
        Ok(s) => match CString::new(s) {
            Ok(cs) => cs.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
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
