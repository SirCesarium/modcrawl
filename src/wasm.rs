//! wasm-bindgen bindings for the browser.
//!
//! Mirrors the C FFI surface over in-memory buffers: every export takes a
//! JAR as a `Uint8Array` and returns JSON as a `String`. Errors surface as
//! JS exceptions (`JsError`).

use std::io::Cursor;
use std::result::Result as StdResult;

use wasm_bindgen::prelude::*;

use crate::core::dep;
use crate::core::identify::identify_reader;
use crate::core::metadata;
use crate::error::Result as CoreResult;

#[cfg(feature = "classfile")]
use crate::core::classfile;

fn to_js(result: CoreResult<String>) -> StdResult<String, JsError> {
    result.map_err(|e| JsError::new(&e.to_string()))
}

/// Detect the mod/plugin type from a ZIP/JAR byte buffer.
///
/// # Errors
///
/// Returns a JS exception when the data is not a valid ZIP archive.
#[wasm_bindgen]
pub fn identify(data: &[u8]) -> StdResult<String, JsError> {
    let mut cursor = Cursor::new(data);

    to_js(identify_reader(&mut cursor).map(|m| m.to_string()))
}

/// Read and parse metadata from a ZIP/JAR byte buffer, returning JSON.
///
/// # Errors
///
/// Returns a JS exception when the JAR is invalid or its metadata cannot
/// be parsed.
#[wasm_bindgen]
pub fn metadata_json(data: &[u8]) -> StdResult<String, JsError> {
    let mut cursor = Cursor::new(data);

    to_js(
        metadata::read_metadata_reader(&mut cursor)
            .and_then(|m| serde_json::to_string(&m).map_err(Into::into)),
    )
}

/// Analyze dependencies from a ZIP/JAR byte buffer, returning JSON.
///
/// `include_jij` controls whether embedded JAR-in-JAR entries appear in
/// the output.
///
/// # Errors
///
/// Returns a JS exception when the JAR is invalid or cannot be parsed.
#[wasm_bindgen]
pub fn deps_json(data: &[u8], include_jij: bool) -> StdResult<String, JsError> {
    let mut cursor = Cursor::new(data);

    to_js(
        dep::analyze_reader(&mut cursor, include_jij)
            .and_then(|r| serde_json::to_string(&r).map_err(Into::into)),
    )
}

/// List all .class files from a ZIP/JAR byte buffer, returning JSON.
///
/// # Errors
///
/// Returns a JS exception when the JAR is invalid.
#[cfg(feature = "classfile")]
#[wasm_bindgen]
pub fn classes_json(data: &[u8]) -> StdResult<String, JsError> {
    let mut cursor = Cursor::new(data);

    to_js(
        classfile::list_classes_reader(&mut cursor)
            .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
    )
}

/// Search the constant pool of all .class files in a ZIP/JAR byte buffer,
/// returning JSON.
///
/// # Errors
///
/// Returns a JS exception when the JAR is invalid.
#[cfg(feature = "classfile")]
#[wasm_bindgen]
pub fn grep_json(data: &[u8], pattern: &str) -> StdResult<String, JsError> {
    let mut cursor = Cursor::new(data);

    to_js(
        classfile::grep_reader(&mut cursor, pattern)
            .and_then(|matches| serde_json::to_string(&matches).map_err(Into::into)),
    )
}

/// Extract mixin targets from a ZIP/JAR byte buffer, returning JSON.
///
/// # Errors
///
/// Returns a JS exception when the JAR is invalid.
#[cfg(feature = "classfile")]
#[wasm_bindgen]
pub fn mixins_json(data: &[u8]) -> StdResult<String, JsError> {
    let mut cursor = Cursor::new(data);

    to_js(
        classfile::mixins_reader(&mut cursor)
            .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
    )
}

/// Find duplicate class entries across multiple ZIP/JAR byte buffers,
/// returning JSON.
///
/// `names` and `jars` are parallel arrays: `names[i]` labels `jars[i]`.
///
/// # Errors
///
/// Returns a JS exception when any buffer is not a valid ZIP archive, or
/// when the arrays differ in length.
#[cfg(feature = "classfile")]
#[wasm_bindgen]
// wasm-bindgen only converts JS string arrays into an owned `Vec<String>`;
// `&[String]` is not a supported argument type.
#[allow(clippy::needless_pass_by_value)]
pub fn dupes_json(names: Vec<String>, jars: &js_sys::Array) -> StdResult<String, JsError> {
    let mut decoded: Vec<Vec<u8>> = Vec::with_capacity(jars.length() as usize);
    for jar in jars.iter() {
        let bytes = js_sys::Uint8Array::new(&jar).to_vec();
        decoded.push(bytes);
    }

    if names.len() != decoded.len() {
        return Err(JsError::new("names and jars must have the same length"));
    }

    let pairs: Vec<(&str, &[u8])> = names
        .iter()
        .zip(decoded.iter())
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();

    to_js(
        classfile::find_duplicates_buffers(&pairs)
            .and_then(|entries| serde_json::to_string(&entries).map_err(Into::into)),
    )
}
