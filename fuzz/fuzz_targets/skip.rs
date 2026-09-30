#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    assert_eq!(
        jzon::serde_impl::from_slice::<serde::de::IgnoredAny>(data).is_ok(),
        reference_json::from_slice::<serde::de::IgnoredAny>(data).is_ok()
    );
});
