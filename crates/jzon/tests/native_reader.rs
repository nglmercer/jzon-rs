#[path = "support/map_keys.rs"]
mod map_keys;
use jzon::serde_impl::{self as native, Category, ReaderDeserializer};
use serde::{
    de::{IgnoredAny, Visitor},
    Deserialize,
};
use std::{
    cell::Cell,
    collections::BTreeMap,
    fmt,
    io::{self, Read},
};
struct Fragment<'a> {
    bytes: &'a [u8],
    calls: usize,
    interrupt: bool,
}
impl Read for Fragment<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.calls += 1;
        if self.interrupt && self.calls % 3 == 0 {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let n = self.bytes.len().min(out.len()).min(2);
        out[..n].copy_from_slice(&self.bytes[..n]);
        self.bytes = &self.bytes[n..];
        Ok(n)
    }
}
fn fragmented(bytes: &[u8]) -> Fragment<'_> {
    Fragment {
        bytes,
        calls: 0,
        interrupt: true,
    }
}
#[derive(Debug, Deserialize, PartialEq)]
enum E {
    Unit,
    New(u8),
    Tuple(u8, String),
    Struct { x: u8 },
}
#[test]
fn fragmented_values_and_enums() {
    for input in [
        r#"{"x":[1,true,null,"a\u0062c"],"y":{"true":1}}"#,
        r#"[1,-2,3.5,1e2]"#,
        r#""😀""#,
    ] {
        assert_eq!(
            native::from_reader::<_, serde_json::Value>(fragmented(input.as_bytes())).unwrap(),
            native::from_str::<serde_json::Value>(input).unwrap()
        );
    }
    for input in [
        r#""Unit""#,
        r#"{"New":1}"#,
        r#"{"Tuple":[1,"abc"]}"#,
        r#"{"Struct":{"x":1}}"#,
    ] {
        assert_eq!(
            native::from_reader::<_, E>(fragmented(input.as_bytes())).unwrap(),
            serde_json::from_str::<E>(input).unwrap()
        );
    }
    for input in [
        r#"{:1}"#,
        r#"{"\u0074rue":1}"#,
        r#"{"true":1,:2}"#,
        r#"{"true":1,}"#,
    ] {
        assert!(
            native::from_reader::<_, BTreeMap<bool, u8>>(fragmented(input.as_bytes())).is_err()
        );
    }
}
#[test]
fn errors_and_truncated_io() {
    for input in [
        "[1,]",
        "[1",
        "{\"a\":1,}",
        "\"\\q\"",
        "01",
        "-",
        "1e",
        "null x",
        "{\"a\" 1}",
    ] {
        let ours =
            native::from_reader::<_, serde_json::Value>(fragmented(input.as_bytes())).unwrap_err();
        let reference =
            serde_json::from_reader::<_, serde_json::Value>(fragmented(input.as_bytes()))
                .unwrap_err();
        assert_eq!(
            format!("{:?}", ours.classify()),
            format!("{:?}", reference.classify()),
            "{input}: {ours}"
        );
    }
}
#[test]
fn stream_partial_consumption_and_fused_errors() {
    let mut stream =
        ReaderDeserializer::new(fragmented(b"1 2 [3,] 4")).into_iter::<serde_json::Value>();
    assert_eq!(stream.next().unwrap().unwrap(), 1);
    assert_eq!(stream.byte_offset(), 1);
    assert_eq!(stream.next().unwrap().unwrap(), 2);
    assert!(stream.next().unwrap().is_err());
    assert!(stream.next().is_none());
    let mut de = ReaderDeserializer::new(fragmented(b"1 2"));
    assert_eq!(u8::deserialize(&mut de).unwrap(), 1);
    let (source, lookahead) = de.into_parts();
    assert_eq!(lookahead, Some(b' '));
    assert_eq!(source.bytes, b"2");
}
#[test]
fn storage_is_not_document_sized() {
    let input = format!("[{}]", vec!["123"; 100_000].join(","));
    let mut de = ReaderDeserializer::new(input.as_bytes());
    let values = Vec::<u32>::deserialize(&mut de).unwrap();
    de.end().unwrap();
    assert_eq!(values.len(), 100_000);
    assert_eq!(de.peak_token_len(), 3);
    let mut de = ReaderDeserializer::new(input.as_bytes());
    IgnoredAny::deserialize(&mut de).unwrap();
    de.end().unwrap();
    assert_eq!(de.peak_token_len(), 3);
    let deep = format!("{}0{}", "[".repeat(1000), "]".repeat(1000));
    native::from_reader::<_, IgnoredAny>(deep.as_bytes()).unwrap();
}
thread_local! {static CALLS:Cell<usize>=const{Cell::new(0)};}
struct Reject;
impl<'de> Deserialize<'de> for Reject {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        CALLS.with(|c| c.set(c.get() + 1));
        struct RejectVisitor;
        impl<'de> Visitor<'de> for RejectVisitor {
            type Value = Reject;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("reject")
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Reject, E> {
                Err(E::custom("visitor wins"))
            }
        }
        de.deserialize_str(RejectVisitor)
    }
}
struct Failing {
    bytes: &'static [u8],
}
impl Read for Failing {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.bytes.is_empty() {
            Err(io::ErrorKind::Other.into())
        } else {
            out[0] = self.bytes[0];
            self.bytes = &self.bytes[1..];
            Ok(1)
        }
    }
}
#[test]
fn callbacks_once_and_error_precedence() {
    CALLS.with(|c| c.set(0));
    let error = native::from_reader::<_, Reject>(Failing { bytes: b"\"abc\"" })
        .err()
        .unwrap();
    assert_eq!(CALLS.with(|c| c.get()), 1);
    assert_eq!(error.classify(), Category::Data);
    assert!(error.to_string().contains("visitor wins"));
    assert!(
        native::from_reader::<_, String>(Failing { bytes: b"\"abc\"" })
            .unwrap_err()
            .is_io()
    );
}
#[test]
fn raw_ignored_and_byte_models() {
    let raw: Box<serde_json::value::RawValue> =
        native::from_reader(fragmented(br#" { "a" : [1, "\uD800"] } "#)).unwrap();
    assert_eq!(raw.get(), r#"{ "a" : [1, "\uD800"] }"#);
    for input in [br#"{"x":"\q"}"#.as_slice(), br#"[1,]"#, br#"{"a":1,}"#] {
        assert!(native::from_reader::<_, IgnoredAny>(fragmented(input)).is_err());
    }
}

#[test]
fn borrowed_slice_stream() {
    let input = r#""one" "two" 3"#;
    let mut stream = native::Deserializer::from_str(input).into_iter::<&str>();
    assert_eq!(stream.next().unwrap().unwrap(), "one");
    assert_eq!(stream.byte_offset(), 5);
    assert_eq!(stream.next().unwrap().unwrap(), "two");
    assert!(stream.next().unwrap().is_err());
    assert!(stream.next().is_none());
}

#[test]
fn reader_bytes_are_transient_and_match_reference() {
    use map_keys::ByteKey;
    for input in [
        br#"{"abc":1}"#.as_slice(),
        br#"{"a\u0062c":1}"#,
        br#"{"\uD800":1}"#,
        b"{\"\xff\":1}",
    ] {
        let actual = native::from_reader::<_, BTreeMap<ByteKey<true>, u8>>(input).unwrap();
        let expected = serde_json::from_reader::<_, BTreeMap<ByteKey<true>, u8>>(input).unwrap();
        assert_eq!(actual, expected);
        assert!(!actual.keys().next().unwrap().1);
    }
}

#[test]
fn reader_custom_key_visitors() {
    fn compare<T: serde::de::DeserializeOwned + PartialEq + fmt::Debug>(input: &str) {
        assert_eq!(
            native::from_reader::<_, T>(input.as_bytes()).ok(),
            serde_json::from_reader::<_, T>(input.as_bytes()).ok()
        );
    }
    for input in [
        r#"{"Unit":1}"#,
        r#"{"true":1,"false":2}"#,
        r#"{:1}"#,
        r#"{"true":1,:2}"#,
        r#"{"é":1}"#,
    ] {
        compare::<BTreeMap<map_keys::EnumKey, u8>>(input);
        compare::<BTreeMap<map_keys::NewtypeKey, u8>>(input);
        compare::<BTreeMap<map_keys::UnitKey<false>, u8>>(input);
        compare::<BTreeMap<map_keys::UnitKey<true>, u8>>(input);
        compare::<BTreeMap<map_keys::ByteKey<false>, u8>>(input);
    }
}

#[test]
fn reader_budget_recovers_after_visitor_unwind() {
    struct Panics;
    impl<'de> Deserialize<'de> for Panics {
        fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct V;
            impl<'de> Visitor<'de> for V {
                type Value = Panics;
                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("a sequence")
                }
                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    mut access: A,
                ) -> Result<Panics, A::Error> {
                    assert!(access.next_element::<u8>()?.is_none());
                    panic!("visitor unwind");
                }
            }
            de.deserialize_seq(V)
        }
    }
    let input = format!("[] {}0{}", "[".repeat(127), "]".repeat(127));
    let mut de = ReaderDeserializer::new(input.as_bytes());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Panics::deserialize(
            &mut de
        )))
        .is_err()
    );
    serde_json::Value::deserialize(&mut de).unwrap();
    de.end().unwrap();
}

#[test]
fn early_container_visitors_do_not_hide_remaining_input() {
    #[derive(Debug)]
    struct Early;
    impl<'de> Deserialize<'de> for Early {
        fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct V;
            impl<'de> Visitor<'de> for V {
                type Value = Early;
                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("a container")
                }
                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    _access: A,
                ) -> Result<Early, A::Error> {
                    Ok(Early)
                }
                fn visit_map<A: serde::de::MapAccess<'de>>(
                    self,
                    _access: A,
                ) -> Result<Early, A::Error> {
                    Ok(Early)
                }
            }
            de.deserialize_any(V)
        }
    }
    for input in ["[1]", "{\"x\":1}", "[", "{"] {
        assert!(native::from_reader::<_, Early>(fragmented(input.as_bytes())).is_err());
        assert!(native::from_str::<Early>(input).is_err());
        assert!(serde_json::from_reader::<_, Early>(input.as_bytes()).is_err());
    }
    for input in ["[]", "{}"] {
        native::from_reader::<_, Early>(fragmented(input.as_bytes())).unwrap();
    }
}

#[test]
fn streams_validate_primitive_boundaries() {
    for input in [
        "1true",
        "truefalse",
        "nullx",
        "trueé",
        "1 2",
        "[]true",
        "1[]",
        "\"x\"1",
    ] {
        let native = native::Deserializer::from_str(input)
            .into_iter::<serde_json::Value>()
            .next();
        let reader = ReaderDeserializer::new(input.as_bytes())
            .into_iter::<serde_json::Value>()
            .next();
        let reference = serde_json::Deserializer::from_str(input)
            .into_iter::<serde_json::Value>()
            .next();
        assert_eq!(
            native.as_ref().map(Result::is_ok),
            reference.as_ref().map(Result::is_ok),
            "slice {input}"
        );
        assert_eq!(
            reader.as_ref().map(Result::is_ok),
            reference.as_ref().map(Result::is_ok),
            "reader {input}"
        );
    }
}

#[test]
fn ignored_token_errors_keep_their_local_position() {
    for input in [
        r#"{"x":"\q"}"#,
        r#"["\uZZZZ"]"#,
        "[\n truex]",
        "{\"x\": 1e]}",
    ] {
        let reader = native::from_reader::<_, IgnoredAny>(input.as_bytes()).unwrap_err();
        let slice = native::from_str::<IgnoredAny>(input).unwrap_err();
        assert_eq!(reader.classify(), slice.classify(), "{input}");
        assert_eq!(
            (reader.line(), reader.column()),
            (slice.line(), slice.column()),
            "{input}: {reader} vs {slice}"
        );
    }
}
