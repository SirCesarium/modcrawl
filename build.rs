#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::env;

fn main() {
    let crate_dir =
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| panic!("CARGO_MANIFEST_DIR must be set"));
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| panic!("OUT_DIR must be set"));
    cbindgen::generate(&crate_dir)
        .unwrap_or_else(|e| panic!("Failed to generate C header: {e}"))
        .write_to_file(format!("{out_dir}/modcrawl.h"));
}
