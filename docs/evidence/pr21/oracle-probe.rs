use std::collections::BTreeMap;
fn main() {
    let result = serde_json::from_str::<BTreeMap<bool, u8>>(r#"{"é":1}"#);
    assert!(result.is_err());
}
