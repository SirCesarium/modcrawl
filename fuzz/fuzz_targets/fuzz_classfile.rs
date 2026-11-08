#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok((_, cf)) = java_class_rs::parse_classfile(data) {
        for entry in &cf.constant_pool {
            let _ = format!("{entry:?}");
        }
    }
});
