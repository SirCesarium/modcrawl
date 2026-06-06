#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::env;

fn main() {
    let crate_dir =
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| panic!("CARGO_MANIFEST_DIR must be set"));
    cbindgen::generate(&crate_dir)
        .unwrap_or_else(|e| panic!("Failed to generate C header: {e}"))
        .write_to_file("modcrawl.h");
}
