#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    let native = jzon::serde_impl::from_slice::<reference_json::Value>(data);
    let reference = reference_json::from_slice::<reference_json::Value>(data);
    assert_eq!(native.is_ok(), reference.is_ok());
    if let (Ok(a), Ok(b)) = (native, reference) {
        assert_eq!(a, b);
    }
    let native = jzon::serde_impl::from_reader::<_, reference_json::Value>(data);
    let reference = reference_json::from_reader::<_, reference_json::Value>(data);
    assert_eq!(native.is_ok(), reference.is_ok(), "reader {data:?}");
    if let (Ok(a), Ok(b)) = (native, reference) {
        assert_eq!(a, b);
    }
});
