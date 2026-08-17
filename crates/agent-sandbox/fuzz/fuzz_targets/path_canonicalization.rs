#![no_main]
use libfuzzer_sys::fuzz_target;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Just verify it doesn't crash on garbage data
        let _ = Path::new(s);
    }
});
