fn generic<T: serde::Serialize + ?Sized>(v: &T) {
    serde_json::to_string(v).unwrap();
    serde_json::to_string_pretty(v).unwrap();
    serde_json::to_vec(v).unwrap();
    serde_json::to_vec_pretty(v).unwrap();
    serde_json::to_writer(Vec::new(), v).unwrap();
    serde_json::to_writer_pretty(Vec::new(), v).unwrap();
}
fn main() {
    generic("str");
    generic(&[1, 2][..]);
    generic(&42);
    let value = serde_json::json!({"x": 1});
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&value.to_string()).unwrap(),
        value
    );
    #[cfg(feature = "mirrors")]
    {
        let raw: &serde_json::value::RawValue = serde_json::from_str("[1, 2]").unwrap();
        assert_eq!(raw.get(), "[1, 2]");
        let big = "9".repeat(100);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&big)
                .unwrap()
                .to_string(),
            big
        );
        let mut de = serde_json::Deserializer::from_str("null");
        de.disable_recursion_limit();
        let map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(r#"{"b":1,"a":2}"#).unwrap();
        assert_eq!(map.keys().next().unwrap(), "b");
    }
}
