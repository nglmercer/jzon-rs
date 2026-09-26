//! Drop-in `serde_json` replacement routing hot-path functions through the
//! [`serde_impl`](crate::serde_impl) engine.
//!
//! Available as `jzon::compat` with the `compat` cargo feature, or through the
//! standalone [`jzon-rs-compat`](https://crates.io/crates/jzon-rs-compat) crate
//! (usable via a dependency rename) which re-exports this module.
//!
//! # Correctness contract
//!
//! Every function below tries the jzon engine first and, on any engine error,
//! retries with the real `serde_json`. The retry makes both the value and the
//! error authoritative: success values, error messages, line/column positions,
//! and error categories (`is_syntax`/`is_data`/`is_eof`/`is_io`) are always
//! exactly what `serde_json` produces. The fallback only runs on the cold
//! error path, so successful parses pay no double-parse cost.
//!
//! One deliberate exception: float parsing is always correctly rounded
//! (equivalent to `serde_json` with `float_roundtrip`), while default
//! `serde_json` is occasionally off by 1ulp. The engine does not replicate
//! the approximation; enable `float_roundtrip` on both sides for
//! bit-identical parses.

use std::io;

pub use serde_json::{Error, Map, Number, Result, Value};
pub use serde_json::{from_value, to_value};
pub use serde_json::json;
pub use serde_json::{Deserializer, Serializer, StreamDeserializer};

pub mod de    { pub use serde_json::de::*; }
pub mod ser   { pub use serde_json::ser::*; }
pub mod error { pub use serde_json::error::*; }
pub mod map   { pub use serde_json::map::*; }
pub mod value { pub use serde_json::value::*; }

#[inline]
pub fn from_str<'de, T: serde::Deserialize<'de>>(s: &'de str) -> Result<T> {
    match crate::serde_impl::from_str(s) {
        Ok(v) => Ok(v),
        Err(_) => serde_json::from_str(s),
    }
}

#[inline]
pub fn from_slice<'de, T: serde::Deserialize<'de>>(v: &'de [u8]) -> Result<T> {
    match crate::serde_impl::from_slice(v) {
        Ok(v) => Ok(v),
        Err(_) => serde_json::from_slice(v),
    }
}

#[inline]
pub fn from_reader<R: io::Read, T: serde::de::DeserializeOwned>(mut r: R) -> Result<T> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).map_err(Error::io)?;
    from_slice(&buf)
}

#[inline]
pub fn to_string<T: serde::Serialize>(v: &T) -> Result<String> {
    match crate::serde_impl::to_string(v) {
        Ok(s) => Ok(s),
        Err(_) => serde_json::to_string(v),
    }
}

#[inline]
pub fn to_string_pretty<T: serde::Serialize>(v: &T) -> Result<String> {
    serde_json::to_string_pretty(v)
}

#[inline]
pub fn to_vec<T: serde::Serialize>(v: &T) -> Result<Vec<u8>> {
    match crate::serde_impl::to_bytes(v) {
        Ok(b) => Ok(b),
        Err(_) => serde_json::to_vec(v),
    }
}

#[inline]
pub fn to_vec_pretty<T: serde::Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(v)
}

#[inline]
pub fn to_writer<W: io::Write, T: serde::Serialize>(mut w: W, v: &T) -> Result<()> {
    // Buffered on the engine path; the retry below streams exactly like
    // serde_json, reproducing its behavior and error byte-for-byte.
    match crate::serde_impl::to_bytes(v) {
        Ok(bytes) => w.write_all(&bytes).map_err(Error::io),
        Err(_) => serde_json::to_writer(w, v),
    }
}

#[inline]
pub fn to_writer_pretty<W: io::Write, T: serde::Serialize>(w: W, v: &T) -> Result<()> {
    serde_json::to_writer_pretty(w, v)
}

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
    fn roundtrip_via_compat() {
        let u = User { id: 1, name: "Alice".into(), score: 9.5 };
        let json = to_string(&u).unwrap();
        let u2: User = from_str(&json).unwrap();
        assert_eq!(u, u2);
    }

    #[test]
    fn value_roundtrip() {
        let v: Value = from_str(r#"{"key": 42}"#).unwrap();
        assert_eq!(v["key"], Value::Number(Number::from(42)));
    }

    #[test]
    fn matches_serde_json_output() {
        let v = vec![1u64, 2, 3];
        assert_eq!(to_string(&v).unwrap(), serde_json::to_string(&v).unwrap());
    }

    #[test]
    fn from_slice_works() {
        let data = br#"{"id":7,"name":"Bob","score":3.14}"#;
        let u: User = from_slice(data).unwrap();
        assert_eq!(u.id, 7);
        assert_eq!(u.name, "Bob");
    }

    #[test]
    fn to_vec_works() {
        let u = User { id: 2, name: "Carol".into(), score: 1.0 };
        let bytes = to_vec(&u).unwrap();
        let u2: User = from_slice(&bytes).unwrap();
        assert_eq!(u, u2);
    }

    #[test]
    fn from_reader_works() {
        let data = br#"{"id":3,"name":"Dave","score":0.0}"#;
        let cursor = std::io::Cursor::new(data);
        let u: User = from_reader(cursor).unwrap();
        assert_eq!(u.id, 3);
        assert_eq!(u.name, "Dave");
    }

    #[test]
    fn to_writer_works() {
        let u = User { id: 4, name: "Eve".into(), score: 2.718 };
        let mut buf = Vec::new();
        to_writer(&mut buf, &u).unwrap();
        let u2: User = from_slice(&buf).unwrap();
        assert_eq!(u, u2);
    }

    #[test]
    fn pretty_functions_work() {
        let v = vec![1u32, 2, 3];
        let pretty = to_string_pretty(&v).unwrap();
        assert!(pretty.contains('\n'));
        let pretty_bytes = to_vec_pretty(&v).unwrap();
        assert!(pretty_bytes.contains(&b'\n'));
    }

    #[test]
    fn json_macro_works() {
        let v = json!({"hello": "world", "n": 42});
        assert_eq!(v["hello"], Value::String("world".into()));
        assert_eq!(v["n"], Value::Number(Number::from(42)));
    }

    #[test]
    fn from_value_to_value_roundtrip() {
        let u = User { id: 99, name: "Zara".into(), score: 100.0 };
        let v = to_value(&u).unwrap();
        let u2: User = from_value(v).unwrap();
        assert_eq!(u, u2);
    }

    // The fallback contract: every error compat returns — message, position,
    // and category — is byte-identical to what serde_json returns.
    #[test]
    fn errors_match_serde_json_exactly() {
        let inputs = vec![
            "1e999".to_string(),
            "9".repeat(500),
            "{\n  bad}".to_string(),
            "1 2".to_string(),
            "[1,]".to_string(),
            "01".to_string(),
            "\"\\uD83D\"".to_string(),
            "\"\\uD83D\\uD00A\"".to_string(),
            ".5".to_string(),
            "-.5".to_string(),
            "[".repeat(200),
            "".to_string(),
            "{".to_string(),
        ];
        for input in &inputs {
            let ours = from_str::<Value>(input).map_err(|e| e.to_string());
            let theirs = serde_json::from_str::<Value>(input).map_err(|e| e.to_string());
            assert_eq!(ours, theirs, "input: {input:?}");

            let ours = from_slice::<Value>(input.as_bytes()).map_err(|e| e.to_string());
            let theirs =
                serde_json::from_slice::<Value>(input.as_bytes()).map_err(|e| e.to_string());
            assert_eq!(ours, theirs, "from_slice input: {input:?}");

            let ours = from_reader::<_, Value>(std::io::Cursor::new(input.as_bytes()))
                .map_err(|e| e.to_string());
            assert_eq!(ours, theirs, "from_reader input: {input:?}");
        }
    }

    #[test]
    fn error_positions_and_categories_are_authoritative() {
        let e = from_str::<Value>("{\n  bad}").unwrap_err();
        assert_eq!((e.line(), e.column()), (2, 3));
        assert!(e.is_syntax());

        let e = from_str::<u8>("256").unwrap_err();
        assert_eq!(
            e.to_string(),
            "invalid value: integer `256`, expected u8 at line 1 column 3"
        );
        assert!(e.is_data());

        let e = from_str::<Value>("").unwrap_err();
        assert!(e.is_eof());

        let e = from_str::<String>("42").unwrap_err();
        assert_eq!(
            e.to_string(),
            "invalid type: integer `42`, expected a string at line 1 column 2"
        );

        // Success values agree too.
        let v: Value = from_str("[1, \"two\", null, true]").unwrap();
        assert_eq!(
            v,
            serde_json::from_str::<Value>("[1, \"two\", null, true]").unwrap()
        );
    }

    #[test]
    fn serialize_errors_match_serde_json() {
        use std::collections::BTreeMap;
        let mut m: BTreeMap<Vec<u8>, &str> = BTreeMap::new();
        m.insert(vec![1], "x");
        let ours = to_string(&m).map_err(|e| e.to_string());
        let theirs = serde_json::to_string(&m).map_err(|e| e.to_string());
        assert_eq!(ours, theirs);

        // Fallback streams exactly like serde_json, including partial writes.
        let mut ours = Vec::new();
        let ours_err = to_writer(&mut ours, &m).map_err(|e| e.to_string());
        let mut theirs_buf = Vec::new();
        let theirs_err = serde_json::to_writer(&mut theirs_buf, &m).map_err(|e| e.to_string());
        assert_eq!(ours_err.unwrap_err(), theirs_err.unwrap_err());
        assert_eq!(ours, theirs_buf);
    }

    #[test]
    fn io_errors_are_io_category() {
        struct FailReader;
        impl io::Read for FailReader {
            fn read(&mut self, _b: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "boom"))
            }
        }
        let e = from_reader::<_, Value>(FailReader).unwrap_err();
        assert!(e.is_io());
    }

    #[test]
    fn raw_value_through_compat() {
        use serde_json::value::RawValue;
        let input = r#"{"a": [1, 2]}"#;
        let v: Box<RawValue> = from_str(input).unwrap();
        assert_eq!(v.get(), input);
        assert_eq!(to_string(&v).unwrap(), input);
    }

    // The `raw_value` flag exposes `RawValue` through compat's own re-export.
    #[cfg(feature = "raw_value")]
    #[test]
    fn raw_value_reexport_available() {
        use crate::compat::value::RawValue;
        let v: Box<RawValue> = from_str("[1, 2]").unwrap();
        assert_eq!(v.get(), "[1, 2]");
    }

    // The `arbitrary_precision` flag keeps big numbers exact end to end.
    #[cfg(feature = "arbitrary_precision")]
    #[test]
    fn arb_precision_exact_through_compat() {
        let digits = "9".repeat(300);
        let v: Value = from_str(&digits).unwrap();
        match &v {
            Value::Number(n) => assert_eq!(n.as_str(), digits),
            other => panic!("expected Number, got {other:?}"),
        }
        assert_eq!(to_string(&v).unwrap(), digits);
        let f: Value = from_str("1.5").unwrap();
        assert_eq!(f, serde_json::from_str::<Value>("1.5").unwrap());
    }

    // The `preserve_order` flag switches `Map` to insertion ordering.
    #[cfg(feature = "preserve_order")]
    #[test]
    fn preserve_order_keeps_insertion_order() {
        let v: Value = from_str(r#"{"b":1,"a":2,"c":3}"#).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, ["b", "a", "c"]);
        let mut m = Map::new();
        m.insert("b".to_string(), Value::from(1));
        m.insert("a".to_string(), Value::from(2));
        let keys: Vec<&str> = m.keys().map(String::as_str).collect();
        assert_eq!(keys, ["b", "a"]);
    }

    // The `unbounded_depth` flag lifts the 128-level recursion limit.
    #[cfg(feature = "unbounded_depth")]
    #[test]
    fn unbounded_depth_parses_deep_input() {
        let input = format!("{}{}", "[".repeat(300), "]".repeat(300));
        let v: Value = from_str(&input).unwrap();
        assert!(v.is_array());
    }

    #[cfg(not(feature = "unbounded_depth"))]
    #[test]
    fn bounded_depth_rejects_deep_input_like_serde_json() {
        let input = format!("{}{}", "[".repeat(300), "]".repeat(300));
        let ours = from_str::<Value>(&input).map_err(|e| e.to_string());
        let theirs = serde_json::from_str::<Value>(&input).map_err(|e| e.to_string());
        assert_eq!(ours, theirs);
    }
}
