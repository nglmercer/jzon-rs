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
        let mut writer_bytes = Vec::new();
        jzon::serde_impl::to_writer(&mut writer_bytes, &value).unwrap();
        assert_eq!(writer_bytes, reference_bytes);
        let mut reusable = jzon::serde_impl::Serializer::with_capacity(bytes.len());
        reusable.serialize(&value).unwrap();
        assert_eq!(reusable.buffer(), reference_bytes);
        reusable.clear();
        reusable.serialize(&value).unwrap();
        assert_eq!(reusable.into_inner(), reference_bytes);
        assert_eq!(
            reference_json::from_slice::<reference_json::Value>(&bytes).unwrap(),
            reference_json::from_slice::<reference_json::Value>(&reference_bytes).unwrap()
        );
    }
});
