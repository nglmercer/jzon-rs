#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    if let Ok(value) = reference_json::from_slice::<reference_json::Value>(data) {
        let bytes = jzon::serde_impl::to_bytes(&value).unwrap();
        let reference_bytes = reference_json::to_vec(&value).unwrap();
        assert_eq!(bytes, reference_bytes);
        assert_eq!(
            reference_json::from_slice::<reference_json::Value>(&bytes).unwrap(),
            reference_json::from_slice::<reference_json::Value>(&reference_bytes).unwrap()
        );
    }
});
