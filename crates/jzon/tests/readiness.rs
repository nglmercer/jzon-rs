use jzon_serde as native;

#[test]
fn tuple_completion() {
    for input in ["[1]", "[1,2]", "[1 2]", "[1", "[1,", "[1,]"] {
        let reference = serde_json::from_str::<(u8,)>(input);
        let candidate = native::from_str::<(u8,)>(input);
        assert_eq!(candidate.is_ok(), reference.is_ok(), "{input}");
    }
}

#[test]
fn exactly_one_scalar() {
    for input in [
        r#""""#,
        r#""a""#,
        r#""ab""#,
        r#""é""#,
        r#""éx""#,
        r#""\uD83D\uDE00""#,
        r#""a\u0301""#,
    ] {
        assert_eq!(
            native::from_str::<char>(input).ok(),
            serde_json::from_str::<char>(input).ok(),
            "{input}"
        );
    }
}

#[derive(serde::Deserialize)]
struct Keep {
    keep: u8,
}

#[test]
fn ignored_strings_validate() {
    for input in [
        br#"{"keep":1,"ignored":"\q"}"#.as_slice(),
        b"{\"keep\":1,\"ignored\":\"\t\"}",
        br#"{"keep":1,"ignored":"\uZZZZ"}"#,
        br#"{"keep":1,"ignored":[1,]}"#,
    ] {
        assert!(serde_json::from_slice::<Keep>(input).is_err());
        assert!(native::from_slice::<Keep>(input).is_err(), "{input:?}");
    }
    assert_eq!(
        native::from_str::<Keep>(r#"{"keep":1,"ignored":["ok",{}]}"#)
            .unwrap()
            .keep,
        1
    );
}

#[test]
fn escaped_capacity_is_local() {
    let input = format!("[\"\\n\",\"{}\"]", "a".repeat(100_000));
    let values: Vec<String> = native::from_str(&input).unwrap();
    assert!(values[0].capacity() < 1024, "{}", values[0].capacity());
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct Pair(u8, u8);
#[derive(Debug, serde::Deserialize, PartialEq)]
enum Enum {
    Pair(u8, u8),
    Unit,
    New(u8),
}

#[test]
fn container_matrix() {
    macro_rules! check {
        ($ty:ty, $inputs:expr) => {
            for input in $inputs {
                let reference = serde_json::from_str::<$ty>(input);
                let candidate = native::from_str::<$ty>(input);
                assert_eq!(
                    candidate.is_ok(),
                    reference.is_ok(),
                    "{}: {input}",
                    stringify!($ty)
                );
                if let (Ok(a), Ok(b)) = (candidate, reference) {
                    assert_eq!(a, b);
                }
            }
        };
    }
    check!(
        [u8; 2],
        ["[1,2]", "[1]", "[1,2,3]", "[1,2,]", "[1,2", "[1 2]"]
    );
    check!(((), (u8,)), ["[null,[1]]", "[null,[1,2]]", "[null,[1]]0"]);
    check!(Pair, ["[1,2]", "[1]", "[1,2,3]", "[1,2,]"]);
    check!(
        Enum,
        [
            r#"{"Pair":[1,2]}"#,
            r#"{"Pair":[1,2,3]}"#,
            r#"{"Unit":null,"New":1}"#,
            r#"{"New":1}"#,
            r#"{"New":1,}"#
        ]
    );
    check!(std::collections::BTreeMap<String,u8>, [r#"{}"#, r#"{"a":1}"#, r#"{"a":1,}"#, r#"{"a":1 "b":2}"#, r#"{"a":}"#]);
}

#[derive(Debug, PartialEq)]
struct Early;
impl<'de> serde::Deserialize<'de> for Early {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Early;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("empty sequence")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, _: A) -> Result<Early, A::Error> {
                Ok(Early)
            }
        }
        de.deserialize_seq(Visitor)
    }
}
#[test]
fn early_visitors_must_finish() {
    for input in ["[]", "[1]", "[", "[1,]", "[]0"] {
        assert_eq!(
            native::from_str::<Early>(input).is_ok(),
            serde_json::from_str::<Early>(input).is_ok(),
            "{input}"
        );
    }
}

#[test]
fn raw_and_ignored_have_distinct_unicode_contracts() {
    for input in [
        r#""\uD800""#,
        r#""\uDC00""#,
        r#""\uZZZZ""#,
        r#""\q""#,
        "\"\t\"",
        r#"[1,]"#,
        r#"{"a":[null]}"#,
    ] {
        assert_eq!(
            native::from_str::<serde::de::IgnoredAny>(input).is_ok(),
            serde_json::from_str::<serde::de::IgnoredAny>(input).is_ok(),
            "ignored {input}"
        );
        assert_eq!(
            native::from_str::<Box<serde_json::value::RawValue>>(input).is_ok(),
            serde_json::from_str::<Box<serde_json::value::RawValue>>(input).is_ok(),
            "raw {input}"
        );
        assert_eq!(
            native::from_str::<&serde_json::value::RawValue>(input).is_ok(),
            serde_json::from_str::<&serde_json::value::RawValue>(input).is_ok(),
            "borrowed raw {input}"
        );
    }
    let input = " \n {\"a\" : [1, 2]} \t";
    let raw: &serde_json::value::RawValue = native::from_str(input).unwrap();
    assert_eq!(
        raw.get(),
        serde_json::from_str::<&serde_json::value::RawValue>(input)
            .unwrap()
            .get()
    );
    assert!(native::from_slice::<serde::de::IgnoredAny>(b"\"\xff\"").is_ok());
    assert!(native::from_slice::<String>(b"\"\xff\"").is_err());
}

#[test]
fn integer_boundaries() {
    macro_rules! check {
        ($ty:ty, $inputs:expr) => {
            for input in $inputs {
                assert_eq!(
                    native::from_str::<$ty>(input).ok(),
                    serde_json::from_str::<$ty>(input).ok(),
                    "{}: {input}",
                    stringify!($ty)
                );
            }
        };
    }
    check!(u8, ["0", "255", "256", "-1", "-0", "1.0", "01", "1e0"]);
    check!(i8, ["-128", "127", "128", "-129", "-0", "1.0"]);
    check!(u64, ["18446744073709551615", "18446744073709551616", "-0"]);
    check!(
        i64,
        [
            "-9223372036854775808",
            "9223372036854775807",
            "9223372036854775808"
        ]
    );
    check!(
        u128,
        [
            "340282366920938463463374607431768211455",
            "340282366920938463463374607431768211456",
            "-0"
        ]
    );
    check!(
        i128,
        [
            "-170141183460469231731687303715884105728",
            "170141183460469231731687303715884105727",
            "170141183460469231731687303715884105728"
        ]
    );
}

#[test]
fn recursion_boundaries_match_reference() {
    for depth in [126, 127, 128, 129, 150] {
        let input = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
        assert_eq!(
            native::from_str::<serde_json::Value>(&input).is_ok(),
            serde_json::from_str::<serde_json::Value>(&input).is_ok(),
            "depth {depth}"
        );
    }
}

#[test]
fn unsized_native_api() {
    fn serialize<T: serde::Serialize + ?Sized>(v: &T) {
        native::to_string(v).unwrap();
        native::to_bytes(v).unwrap();
        native::to_writer(Vec::new(), v).unwrap();
    }
    serialize("text");
    serialize(&[1, 2][..]);
}

#[test]
fn reuse_buffer_and_failure_output() {
    let mut buffer = Vec::with_capacity(1024);
    let ptr = buffer.as_ptr();
    native::to_bytes_in("text", &mut buffer).unwrap();
    assert_eq!(buffer, b"\"text\"");
    assert_eq!(ptr, buffer.as_ptr());
    buffer.clear();
    native::to_bytes_in(&[1u8, 2], &mut buffer).unwrap();
    assert_eq!(buffer, b"[1,2]");
    assert_eq!(ptr, buffer.as_ptr());
}

#[derive(Debug, PartialEq)]
struct Trace(&'static str);
impl<'de> serde::Deserialize<'de> for Trace {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Trace;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("scalar")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut access: A,
            ) -> Result<Trace, A::Error> {
                while access.next_entry::<String, String>()?.is_some() {}
                Ok(Trace("map"))
            }
            fn visit_i64<E>(self, _: i64) -> Result<Trace, E> {
                Ok(Trace("i64"))
            }
            fn visit_u64<E>(self, _: u64) -> Result<Trace, E> {
                Ok(Trace("u64"))
            }
            fn visit_f64<E>(self, _: f64) -> Result<Trace, E> {
                Ok(Trace("f64"))
            }
            fn visit_borrowed_str<E>(self, _: &'de str) -> Result<Trace, E> {
                Ok(Trace("borrowed"))
            }
            fn visit_str<E>(self, _: &str) -> Result<Trace, E> {
                Ok(Trace("transient"))
            }
            fn visit_string<E>(self, _: String) -> Result<Trace, E> {
                Ok(Trace("owned"))
            }
        }
        de.deserialize_any(V)
    }
}
#[test]
fn scalar_callback_traces() {
    for input in ["1", "-1", "-0", "1.5", r#""plain""#, r#""escaped\n""#] {
        assert_eq!(
            native::from_str::<Trace>(input).unwrap(),
            serde_json::from_str::<Trace>(input).unwrap()
        );
    }
}

#[derive(Debug, PartialEq)]
struct Bytes(Vec<u8>);
impl<'de> serde::Deserialize<'de> for Bytes {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Bytes;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bytes")
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Bytes, E> {
                Ok(Bytes(v.to_vec()))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Bytes, A::Error> {
                let mut v = Vec::new();
                while let Some(x) = a.next_element()? {
                    v.push(x);
                }
                Ok(Bytes(v))
            }
        }
        de.deserialize_bytes(V)
    }
}
#[test]
fn byte_model() {
    for input in [
        b"\"\xff\t\"".as_slice(),
        br#""\uD800""#,
        br#""\uD800\u0061""#,
        br#""\uD800\n""#,
        br#""\uD83D\uDE00""#,
        br#""\uDC00""#,
        br#""a\uD800\uD801\uDC00""#,
        br#""\q""#,
        b"[1,255]",
    ] {
        assert_eq!(
            native::from_slice::<Bytes>(input).ok(),
            serde_json::from_slice::<Bytes>(input).ok(),
            "{input:?}"
        );
    }
}
#[test]
fn map_key_grammar() {
    for input in [
        r#"{"1":1}"#,
        r#"{"01":1}"#,
        r#"{"+1":1}"#,
        r#"{" 1":1}"#,
        r#"{"1 ":1}"#,
        r#"{"1.0":1}"#,
        r#"{"\u0031":1}"#,
    ] {
        type Map = std::collections::BTreeMap<u8, u8>;
        assert_eq!(
            native::from_str::<Map>(input).ok(),
            serde_json::from_str::<Map>(input).ok(),
            "{input}"
        );
    }
}

#[test]
fn whitespace_bulk_blocks_reject_other_controls() {
    for control in (0u8..=32).filter(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r')) {
        for count in [1, 7, 8, 9, 16] {
            let mut input = b"null".to_vec();
            input.extend(std::iter::repeat_n(control, count));
            assert!(serde_json::from_slice::<serde::de::IgnoredAny>(&input).is_err());
            assert!(
                native::from_slice::<serde::de::IgnoredAny>(&input).is_err(),
                "control {control}, count {count}"
            );
            assert!(native::from_slice::<Box<serde_json::value::RawValue>>(&input).is_err());
        }
    }
}

#[test]
fn float_bits_match_equivalent_configuration() {
    for input in [
        "0",
        "-0",
        "0.0",
        "-0.0",
        "1.234567890123456789",
        "1.00000000000000011102230246251565404236316680908203125",
        "18446744073709551616",
        "1234567890123456789012345.6789",
        "1e-324",
        "-1e-999",
        "1e309",
        "0e99999999999",
        "1e-99999999999",
        "1.7976931348623157e308",
        "2.2250738585072014e-308",
    ] {
        let a = native::from_str::<f64>(input).map(f64::to_bits).ok();
        let b = serde_json::from_str::<f64>(input).map(f64::to_bits).ok();
        assert_eq!(a, b, "f64 {input}");
        let a = native::from_str::<f32>(input).map(f32::to_bits).ok();
        let b = serde_json::from_str::<f32>(input).map(f32::to_bits).ok();
        assert_eq!(a, b, "f32 {input}");
    }
}

#[test]
fn iterative_ignored_and_raw_follow_upstream_depth_policy() {
    for depth in [126, 127, 128, 300, 1000] {
        let input = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
        assert_eq!(
            native::from_str::<serde::de::IgnoredAny>(&input).is_ok(),
            serde_json::from_str::<serde::de::IgnoredAny>(&input).is_ok()
        );
        assert_eq!(
            native::from_str::<&serde_json::value::RawValue>(&input).is_ok(),
            serde_json::from_str::<&serde_json::value::RawValue>(&input).is_ok()
        );
    }
}

struct WriteFailure {
    panic: bool,
}
impl serde::Serialize for WriteFailure {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(2))?;
        seq.serialize_element(&1u8)?;
        if self.panic {
            panic!("user panic");
        }
        Err(serde::ser::Error::custom("user failure"))
    }
}
#[test]
fn reusable_buffer_restores_partial_output_on_error_and_panic() {
    for panic in [false, true] {
        let mut output = Vec::with_capacity(1024);
        output.extend_from_slice(b"prefix");
        let original = output.as_ptr();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            native::to_bytes_in(&WriteFailure { panic }, &mut output)
        }));
        if panic {
            assert!(result.is_err());
        } else {
            assert!(result.unwrap().is_err());
        }
        assert_eq!(output, b"prefix[1");
        assert_eq!(original, output.as_ptr());
        output.clear();
        native::to_bytes_in(&2u8, &mut output).unwrap();
        assert_eq!(output, b"2");
    }
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct Named {
    a: u8,
    b: String,
}
#[derive(Debug, serde::Deserialize, PartialEq)]
enum NamedEnum {
    Named { a: u8, b: String },
}
#[test]
fn structs_accept_reference_sequence_representation() {
    for input in [
        r#"[1,"text"]"#,
        r#"{"a":1,"b":"text"}"#,
        r#"[1,"text",2]"#,
        r#"[1]"#,
    ] {
        assert_eq!(
            native::from_str::<Named>(input).ok(),
            serde_json::from_str::<Named>(input).ok()
        );
    }
    let input = r#"{"Named":[1,"text"]}"#;
    assert_eq!(
        native::from_str::<NamedEnum>(input).ok(),
        serde_json::from_str::<NamedEnum>(input).ok()
    );
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Attributes {
    #[serde(alias = "identifier")]
    id: u8,
    #[serde(default)]
    enabled: bool,
    #[serde(flatten)]
    rest: std::collections::BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Deny {
    id: u8,
}
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "kind")]
enum Internal {
    Data { id: u8 },
    Unit,
}
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "kind", content = "data")]
enum Adjacent {
    Data(u8),
    Unit,
}
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(untagged)]
enum Untagged {
    Number(u8),
    Text(String),
}
#[test]
fn derived_attribute_and_enum_matrix() {
    macro_rules! check {
        ($ty:ty,$inputs:expr) => {
            for input in $inputs {
                assert_eq!(
                    native::from_str::<$ty>(input).ok(),
                    serde_json::from_str::<$ty>(input).ok(),
                    "{} {input}",
                    stringify!($ty)
                );
            }
        };
    }
    check!(
        Attributes,
        [
            r#"{"identifier":1,"extra":[2]}"#,
            r#"{"id":1,"identifier":2}"#,
            r#"{"id":1,"enabled":true}"#
        ]
    );
    check!(
        Deny,
        [r#"{"id":1}"#, r#"{"id":1,"extra":2}"#, r#"{"id":1,"id":2}"#]
    );
    check!(
        Internal,
        [
            r#"{"kind":"Data","id":1}"#,
            r#"{"id":1,"kind":"Data"}"#,
            r#"{"kind":"Unit"}"#
        ]
    );
    check!(
        Adjacent,
        [
            r#"{"kind":"Data","data":1}"#,
            r#"{"data":1,"kind":"Data"}"#,
            r#"{"kind":"Unit"}"#
        ]
    );
    check!(Untagged, ["1", r#""text""#, "256", "null"]);
    type EnumKeys = std::collections::BTreeMap<EnumKey, u8>;
    check!(EnumKeys, [r#"{"A":1}"#, r#"{"B":2}"#, r#"{"C":1}"#]);
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
enum EnumKey {
    A,
    B,
}

#[test]
fn scalar_serialization_matches_reference() {
    for value in [
        0.0,
        -0.0,
        1.2345678901234567,
        1e-300,
        1e300,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert_eq!(
            native::to_string(&value).unwrap(),
            serde_json::to_string(&value).unwrap()
        );
    }
    for value in [0.0f32, -0.0, 1.234567, 1e-30, f32::NAN, f32::INFINITY] {
        assert_eq!(
            native::to_string(&value).unwrap(),
            serde_json::to_string(&value).unwrap()
        );
    }
    assert_eq!(
        native::from_str::<&str>(r#""borrowed""#).unwrap(),
        "borrowed"
    );
    assert!(native::from_str::<&str>(r#""escaped\n""#).is_err());
    assert_eq!(native::from_str::<Option<bool>>("null").unwrap(), None);
    assert_eq!(
        native::from_str::<Option<bool>>("true").unwrap(),
        Some(true)
    );
    native::from_str::<()>("null").unwrap();
}
