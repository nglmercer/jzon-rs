//! Drop-in `serde_json` replacement routing hot-path functions through `jzon`'s
//! SIMD engine.
//!
//! This crate re-exports [`jzon`](https://crates.io/crates/jzon-rs) (feature
//! `compat`). Use it via a **dependency rename** — one line per crate, zero
//! code changes:
//!
//! ```toml
//! [dependencies]
//! serde_json = { package = "jzon-rs-compat", version = "0.3" }
//! ```
//!
//! Types are `serde_json`'s own (re-exported), so values cross freely between
//! renamed crates and third-party crates still on real `serde_json`, and
//! every function falls back to real `serde_json` on engine error, making
//! values, error messages, and line/column positions authoritative.
//!
//! Note: `[patch.crates-io]` cannot express this replacement — cargo silently
//! ignores renamed patches, and a same-name vendored patch is ignored too
//! because this crate itself depends on real `serde_json` (fallback + type
//! re-exports). Rename per crate instead; see the crate README.

pub use jzon::compat::{
    de, error, from_reader, from_slice, from_str, from_value, json, map, ser, to_string,
    to_string_pretty, to_value, to_vec, to_vec_pretty, to_writer, to_writer_pretty, value,
    Deserializer, Error, Map, Number, Result, Serializer, StreamDeserializer, Value,
};

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
