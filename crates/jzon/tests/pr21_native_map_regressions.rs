//! Native entry points deliberately bypass both compatibility facades.
#[path = "support/map_keys.rs"]
mod map_keys;
use map_keys::{ByteKey, EnumKey, NewtypeKey, UnitKey};
use serde::de::{DeserializeOwned, Deserializer, Visitor};
use serde::Deserialize;
use std::{collections::BTreeMap, fmt};

fn compare<T: DeserializeOwned + PartialEq + fmt::Debug>(input: &[u8]) {
    let reference = serde_json::from_slice::<T>(input);
    let native = jzon::serde_impl::from_slice::<T>(input);
    assert_eq!(
        native.is_ok(),
        reference.is_ok(),
        "slice {input:?}: native={native:?}, reference={reference:?}"
    );
    if let (Ok(a), Ok(b)) = (native, reference) {
        assert_eq!(a, b, "slice {input:?}");
    }
    if let Ok(text) = std::str::from_utf8(input) {
        let reference = serde_json::from_str::<T>(text);
        let native = jzon::serde_impl::from_str::<T>(text);
        assert_eq!(
            native.is_ok(),
            reference.is_ok(),
            "str {text:?}: native={native:?}, reference={reference:?}"
        );
        if let (Ok(a), Ok(b)) = (native, reference) {
            assert_eq!(a, b, "str {text:?}");
        }
    }
}

#[test]
fn missing_unit_keys() {
    for input in [
        "{:1}",
        "{ \n :1}",
        "{",
        "{ ",
        "{,}",
        "{null:1}",
        "{\"x\":1}",
        "{}",
    ] {
        compare::<BTreeMap<(), u8>>(input.as_bytes());
    }
}

#[test]
fn enum_key_grammar() {
    for input in [
        r#"{{"New":1}:2}"#,
        r#"{"Unit":1,{"New":1}:2}"#,
        r#"{"Unit":1,:2}"#,
        r#"{"Unit":1,}"#,
        r#"{"Unit":1,"Unit":2}"#,
        r#"{"New":1}"#,
        r#"{"Unit":1,"#,
        r#"{"Unit":1, true:2}"#,
    ] {
        compare::<BTreeMap<EnumKey, u8>>(input.as_bytes());
    }
}

#[test]
fn boolean_key_lexemes() {
    for input in [
        r#"{"true":1,"false":2}"#,
        r#"{"\u0074rue":1}"#,
        r#"{"t\u0072ue":1}"#,
        r#"{"\u0066alse":1}"#,
        r#"{"fa\u006cse":1}"#,
        r#"{"truex":1}"#,
        r#"{"false ":1}"#,
        r#"{" true":1}"#,
        r#"{"true:1}"#,
        r#"{"false"#,
        r#"{"":1}"#,
        r#"{"true":1,false:2}"#,
    ] {
        compare::<BTreeMap<bool, u8>>(input.as_bytes());
        compare::<BTreeMap<NewtypeKey, u8>>(input.as_bytes());
    }
}

#[test]
fn non_ascii_boolean_keys_use_slice_oracle() {
    // 1.0.151's bool-key StrRead error branch violates UTF-8 invariants for
    // a multibyte first character. Exercise native from_str without executing
    // that known-defective oracle branch; pinned from_slice safely rejects.
    for input in [r#"{"é":1}"#, r#"{"true":1,"é":2}"#, r#"{"😀":1}"#] {
        assert!(serde_json::from_slice::<BTreeMap<bool, u8>>(input.as_bytes()).is_err());
        assert!(jzon::serde_impl::from_str::<BTreeMap<bool, u8>>(input).is_err());
        assert!(jzon::serde_impl::from_slice::<BTreeMap<bool, u8>>(input.as_bytes()).is_err());
        assert!(serde_json::from_slice::<BTreeMap<NewtypeKey, u8>>(input.as_bytes()).is_err());
        assert!(jzon::serde_impl::from_str::<BTreeMap<NewtypeKey, u8>>(input).is_err());
        assert!(
            jzon::serde_impl::from_slice::<BTreeMap<NewtypeKey, u8>>(input.as_bytes()).is_err()
        );
    }
}

#[test]
fn byte_key_callbacks_and_model() {
    for input in [
        br#"{"abc":1}"#.as_slice(),
        br#"{"a\u0062c":1}"#,
        br#"{"\uD800":1}"#,
        br#"{"\uDC00":1}"#,
        br#"{"\uD83D\uDE00":1}"#,
        b"{\"\xff\":1}",
        b"{\"\t\":1}",
        br#"{"\q":1}"#,
        br#"{"abc":1,[1]:2}"#,
        br#"{"abc":1,"def":2}"#,
        br#"{"abc":1,"#,
    ] {
        compare::<BTreeMap<ByteKey<false>, u8>>(input);
        compare::<BTreeMap<ByteKey<true>, u8>>(input);
    }
    for (input, borrowed) in [(r#"{"abc":1}"#, true), (r#"{"a\u0062c":1}"#, false)] {
        let result: BTreeMap<ByteKey<false>, u8> = jzon::serde_impl::from_str(input).unwrap();
        assert_eq!(
            result.keys().next().unwrap(),
            &ByteKey(b"abc".to_vec(), borrowed)
        );
    }
    let borrowed: BTreeMap<&[u8], u8> = jzon::serde_impl::from_str(r#"{"abc":1}"#).unwrap();
    assert_eq!(borrowed.keys().next().copied(), Some(b"abc".as_slice()));
}

thread_local! { static KEY_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UnitProbe;
impl<'de> Deserialize<'de> for UnitProbe {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        KEY_CALLS.with(|calls| calls.set(calls.get() + 1));
        struct UnitVisitor;
        impl<'de> Visitor<'de> for UnitVisitor {
            type Value = UnitProbe;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a unit key")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(UnitProbe)
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Self::Value, E> {
                Ok(UnitProbe)
            }
        }
        de.deserialize_unit(UnitVisitor)
    }
}

#[test]
fn invalid_key_never_invokes_seed() {
    for (input, calls) in [
        ("{:1}", 0),
        ("{ \t :1}", 0),
        (r#"{"ok":1,:2}"#, 1),
        (r#"{"ok":1, null:2}"#, 1),
        (r#"{"ok":1,"#, 1),
    ] {
        KEY_CALLS.with(|n| n.set(0));
        assert!(serde_json::from_str::<BTreeMap<UnitProbe, u8>>(input).is_err());
        assert_eq!(KEY_CALLS.with(|n| n.get()), calls);
        KEY_CALLS.with(|n| n.set(0));
        assert!(jzon::serde_impl::from_str::<BTreeMap<UnitProbe, u8>>(input).is_err());
        assert_eq!(KEY_CALLS.with(|n| n.get()), calls, "{input}");
        KEY_CALLS.with(|n| n.set(0));
        assert!(jzon::serde_impl::from_slice::<BTreeMap<UnitProbe, u8>>(input.as_bytes()).is_err());
        assert_eq!(KEY_CALLS.with(|n| n.get()), calls, "{input}");
    }
}

#[test]
fn custom_unit_visitor_consumes_quoted_key() {
    compare::<BTreeMap<UnitProbe, u8>>(br#"{"ok":1,"next":2}"#);
    compare::<BTreeMap<UnitKey<false>, u8>>(br#"{"ok":1,"next":2}"#);
    compare::<BTreeMap<UnitKey<true>, u8>>(br#"{"ok":1,"next":2}"#);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct BoolProbe(bool);
impl<'de> Deserialize<'de> for BoolProbe {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct BoolVisitor;
        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = BoolProbe;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a boolean key")
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                KEY_CALLS.with(|calls| calls.set(calls.get() + 1));
                Ok(BoolProbe(value))
            }
        }
        de.deserialize_bool(BoolVisitor)
    }
}

#[test]
fn bool_callback_requires_complete_lexeme() {
    for (input, expected_calls) in [
        (r#"{"true":1,"false":2}"#, 2),
        (r#"{"truex":1}"#, 0),
        (r#"{"true:1}"#, 0),
        (r#"{"t\u0072ue":1}"#, 0),
        (r#"{"\u0074rue":1}"#, 0),
    ] {
        KEY_CALLS.with(|n| n.set(0));
        let reference = serde_json::from_str::<BTreeMap<BoolProbe, u8>>(input);
        assert_eq!(KEY_CALLS.with(|n| n.get()), expected_calls);
        KEY_CALLS.with(|n| n.set(0));
        let native = jzon::serde_impl::from_str::<BTreeMap<BoolProbe, u8>>(input);
        assert_eq!(native.ok(), reference.ok(), "{input}");
        assert_eq!(KEY_CALLS.with(|n| n.get()), expected_calls);
        KEY_CALLS.with(|n| n.set(0));
        let _ = jzon::serde_impl::from_slice::<BTreeMap<BoolProbe, u8>>(input.as_bytes());
        assert_eq!(KEY_CALLS.with(|n| n.get()), expected_calls);
    }
}
