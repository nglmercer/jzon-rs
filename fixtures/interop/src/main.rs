fn main() {
    let upstream: serde_json::Value = real_json_user::value();
    let json = serde_json::to_string(&upstream).unwrap();
    let facade = serde_json::from_str::<serde_json::Value>(&json).unwrap();
    assert_eq!(real_json_user::identity(facade), upstream);
    // These are enabled only by the downstream real-json-user crate.
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
}
