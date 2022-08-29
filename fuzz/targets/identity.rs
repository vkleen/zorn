#![no_main]
use libfuzzer_sys::fuzz_target;
use zorn_core::identity::ZornIdentity;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data).map(|s| s.trim_end()) {
        let _ = s.parse::<ZornIdentity>();
    }
});
