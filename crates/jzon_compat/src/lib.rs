//! Strict dependency-renamable facade for serde_json 1.0.151.
//! All operations and types are upstream re-exports; each callback runs once.
//! Supports std (default) or no_std with alloc. Native performance options are
//! legacy no-ops here; use jzon-rs-serde to select the independent native engine.
#![cfg_attr(not(feature = "std"), no_std)]
pub use serde_json::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct User {
        id: u64,
        name: String,
        score: f64,
    }

    #[test]
    fn shim_roundtrip() {
        let u = User {
            id: 1,
            name: "Alice".into(),
            score: 9.5,
        };
        let json = to_string(&u).unwrap();
        let u2: User = from_str(&json).unwrap();
        assert_eq!(u, u2);
    }

    #[test]
    fn shim_value_and_macro() {
        let v: Value = from_str(r#"{"key": 42}"#).unwrap();
        assert_eq!(v["key"], Value::Number(Number::from(42)));
        let m = json!({"hello": "world"});
        assert_eq!(m["hello"], Value::String("world".into()));
    }

    #[cfg(feature = "arbitrary_precision")]
    #[test]
    fn shim_arb_precision_exact() {
        let digits = "9".repeat(200);
        let v: Value = from_str(&digits).unwrap();
        match &v {
            Value::Number(n) => assert_eq!(n.as_str(), digits),
            other => panic!("expected Number, got {other:?}"),
        }
    }

    #[cfg(feature = "raw_value")]
    #[test]
    fn shim_raw_value_reexport() {
        use crate::value::RawValue;
        let v: Box<RawValue> = from_str(r#"{"a": [1]}"#).unwrap();
        assert_eq!(v.get(), r#"{"a": [1]}"#);
    }

    #[cfg(feature = "preserve_order")]
    #[test]
    fn shim_preserve_order() {
        let v: Value = from_str(r#"{"b":1,"a":2}"#).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, ["b", "a"]);
    }
}
