#![no_main]

use std::io::Cursor;
use libfuzzer_sys::fuzz_target;
use modcrawl::{
    analyze_reader, grep_reader, identify_reader, list_classes_reader, mixins_reader,
    read_metadata_reader,
};

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);
    let _ = identify_reader(&mut cursor);

    cursor.set_position(0);
    let _ = read_metadata_reader(&mut cursor);

    cursor.set_position(0);
    let _ = analyze_reader(&mut cursor, false);

    cursor.set_position(0);
    let _ = list_classes_reader(&mut cursor);

    cursor.set_position(0);
    let _ = grep_reader(&mut cursor, "test");

    cursor.set_position(0);
    let _ = mixins_reader(&mut cursor);
});
