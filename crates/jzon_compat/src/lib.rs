//! Drop-in `serde_json` replacement routing hot-path functions through `jzon`'s
//! SIMD engine.
//!
//! This crate re-exports [`jzon`](https://crates.io/crates/jzon-rs) (feature
//! `compat`); it exists so a single `[patch.crates-io]` line swaps every
//! transitive `serde_json` dependency to jzon with no code changes:
//!
//! ```toml
//! [patch.crates-io]
//! serde_json = { package = "jzon-rs-compat", version = "0.3" }
//! ```

pub use jzon::compat::{
    de, error, from_reader, from_slice, from_str, from_value, json, map, ser, to_string,
    to_string_pretty, to_value, to_vec, to_vec_pretty, to_writer, to_writer_pretty, Deserializer,
    Error, Map, Number, Result, Serializer, StreamDeserializer, Value,
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
}
