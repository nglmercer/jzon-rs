use jzon_serde as native;
#[test]
fn bounded_json_test_suite() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let mut count = 0;
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "json")
            || path.file_name().unwrap() == "manifest.json"
        {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let reference = serde_json::from_slice::<serde_json::Value>(&bytes);
        let candidate = native::from_slice::<serde_json::Value>(&bytes);
        assert_eq!(candidate.is_ok(), reference.is_ok(), "{}", path.display());
        if let (Ok(a), Ok(b)) = (candidate, reference) {
            assert_eq!(a, b, "{}", path.display());
        }
        // i_ cases are implementation-defined; compare only to the pinned
        // oracle. n_/y_ are not applied to destinations like IgnoredAny,
        // whose numeric/Unicode model intentionally differs from Value.
        assert_eq!(
            native::from_slice::<serde::de::IgnoredAny>(&bytes).is_ok(),
            serde_json::from_slice::<serde::de::IgnoredAny>(&bytes).is_ok(),
            "ignored {}",
            path.display()
        );
        let raw = native::from_slice::<Box<serde_json::value::RawValue>>(&bytes);
        let oracle = serde_json::from_slice::<Box<serde_json::value::RawValue>>(&bytes);
        assert_eq!(raw.is_ok(), oracle.is_ok(), "raw {}", path.display());
        if let (Ok(a), Ok(b)) = (raw, oracle) {
            assert_eq!(a.get(), b.get(), "raw {}", path.display());
        }
        count += 1;
    }
    assert!(count >= 300);
}
