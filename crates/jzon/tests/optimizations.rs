#![cfg(feature = "serde")]
use jzon::serde_impl as native;
use serde::{Deserialize, Serialize};
use std::cell::Cell;

#[test]
fn emitted_strings_match_oracle_at_dispatch_boundaries() {
    use jzon::ToJson;
    for len in [0, 1, 7, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129] {
        for marker in ['"', '\\', '\n', '\t', '\0', '\u{1f}', 'é', '🦀'] {
            for position in [0, len / 2, len] {
                let mut value = "a".repeat(len);
                value.insert(position, marker);
                let reference = serde_json::to_vec(&value).unwrap();
                assert_eq!(native::to_bytes(&value).unwrap(), reference);
                assert_eq!(value.to_json_bytes(), reference);
                let mut streamed = Vec::new();
                native::to_writer(&mut streamed, &value).unwrap();
                assert_eq!(streamed, reference);
            }
        }
    }
}

struct BudgetWriter {
    bytes: Vec<u8>,
    budget: usize,
    interrupted: bool,
}
impl std::io::Write for BudgetWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.interrupted {
            self.interrupted = false;
            return Err(std::io::ErrorKind::Interrupted.into());
        }
        if self.bytes.len() == self.budget {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "byte budget",
            ));
        }
        let count = bytes.len().min(2).min(self.budget - self.bytes.len());
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct Tracked<'a> {
    calls: &'a Cell<usize>,
    fail: bool,
}
impl Serialize for Tracked<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            return Err(serde::ser::Error::custom("user failure"));
        }
        serializer.serialize_str("é\n\\\"🦀")
    }
}
struct Composite<'a> {
    calls: &'a Cell<usize>,
    fail: bool,
}
impl Serialize for Composite<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        self.calls.set(self.calls.get() + 1);
        let mut fields = serializer.serialize_struct("Composite", 3)?;
        fields.serialize_field(
            "quo\"te",
            &Tracked {
                calls: self.calls,
                fail: false,
            },
        )?;
        fields.serialize_field(
            "second\n",
            &Tracked {
                calls: self.calls,
                fail: self.fail,
            },
        )?;
        fields.serialize_field("tail", &[1u64, 2, 3])?;
        fields.end()
    }
}

#[test]
fn streaming_io_and_callback_order_match_reference() {
    for fail in [false, true] {
        for budget in 0..100 {
            let a = Cell::new(0);
            let b = Cell::new(0);
            let mut ours = BudgetWriter {
                bytes: Vec::new(),
                budget,
                interrupted: true,
            };
            let mut reference = BudgetWriter {
                bytes: Vec::new(),
                budget,
                interrupted: true,
            };
            let candidate = native::to_writer(&mut ours, &Composite { calls: &a, fail });
            let oracle = serde_json::to_writer(&mut reference, &Composite { calls: &b, fail });
            assert_eq!(candidate.is_ok(), oracle.is_ok(), "budget {budget}");
            assert_eq!(ours.bytes, reference.bytes, "budget {budget}");
            assert_eq!(a.get(), b.get(), "callbacks at budget {budget}");
            if let (Err(candidate), Err(oracle)) = (candidate, oracle) {
                match candidate {
                    native::Error::Io(error) => {
                        assert!(oracle.is_io());
                        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
                        assert_eq!(error.to_string(), oracle.to_string());
                    }
                    error => {
                        assert!(!oracle.is_io());
                        assert_eq!(error.to_string(), oracle.to_string());
                    }
                }
            }
        }
    }
}

#[test]
fn buffered_writer_keeps_its_explicit_contract() {
    let calls = Cell::new(0);
    let mut writer = BudgetWriter {
        bytes: Vec::new(),
        budget: 0,
        interrupted: false,
    };
    let error = native::to_writer_buffered(
        &mut writer,
        &Composite {
            calls: &calls,
            fail: true,
        },
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "user failure");
    assert_eq!(calls.get(), 3);
    assert!(writer.bytes.is_empty());
}

#[test]
fn reusable_serializer_preserves_capacity_and_partial_output() {
    let mut serializer = native::Serializer::with_capacity(2048);
    serializer.serialize("direct unsized str").unwrap();
    assert_eq!(serializer.buffer(), br#""direct unsized str""#);
    serializer.clear();
    serializer.serialize(&[1, 2, 3][..]).unwrap();
    assert_eq!(serializer.buffer(), b"[1,2,3]");
    assert_eq!(serializer.capacity(), 2048);
    serializer.clear();
    let calls = Cell::new(0);
    assert!(serializer
        .serialize(&Composite {
            calls: &calls,
            fail: true
        })
        .is_err());
    assert_eq!(calls.get(), 3);
    assert!(!serializer.buffer().is_empty());
    let partial = serializer.into_inner();
    let mut reused = native::Serializer::from_vec(partial.clone());
    reused.serialize(&42).unwrap();
    assert!(reused.buffer().starts_with(&partial));
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Variant {
    Unit,
    Tuple(u64, String),
    Struct {
        #[serde(rename = "escaped\"key\n")]
        value: String,
    },
}
#[test]
fn dedicated_structs_and_variants_match_reference() {
    for value in [
        Variant::Unit,
        Variant::Tuple(7, "escaped\n".into()),
        Variant::Struct {
            value: "🦀".into()
        },
    ] {
        let expected = serde_json::to_vec(&value).unwrap();
        assert_eq!(native::to_bytes(&value).unwrap(), expected);
        let mut writer = native::Serializer::from_writer(Vec::new());
        writer.serialize(&value).unwrap();
        assert_eq!(writer.into_inner().into_inner(), expected);
        assert_eq!(native::from_slice::<Variant>(&expected).unwrap(), value);
    }
}

struct PanicString;
impl<'de> Deserialize<'de> for PanicString {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PanicString;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string")
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<PanicString, E> {
                panic!("visitor panic");
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}
#[test]
fn scratch_reuses_small_strings_and_releases_oversized_on_unwind() {
    let mut parser = native::Deserializer::from_str(r#""\n" "\t" "borrowed""#);
    assert_eq!(String::deserialize(&mut parser).unwrap(), "\n");
    let capacity = parser.scratch_capacity();
    assert!(capacity > 0);
    assert_eq!(String::deserialize(&mut parser).unwrap(), "\t");
    assert_eq!(parser.scratch_capacity(), capacity);
    let borrowed = <&str>::deserialize(&mut parser).unwrap();
    assert_eq!(borrowed, "borrowed");
    parser.end().unwrap();
    parser.clear_scratch();
    assert_eq!(parser.scratch_capacity(), 0);

    let input = format!("\"{}\\n\" \"next\"", "a".repeat(70000));
    let mut parser = native::Deserializer::from_str(&input);
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        PanicString::deserialize(&mut parser)
    }));
    assert!(panic.is_err());
    assert_eq!(parser.scratch_capacity(), 0);
    assert_eq!(<&str>::deserialize(&mut parser).unwrap(), "next");
    parser.end().unwrap();
    for suffix in [r#"\q"#, r#"\uZZZZ"#] {
        let input = format!("\"{}{}\"", "a".repeat(70000), suffix);
        let mut parser = native::Deserializer::from_str(&input);
        assert!(String::deserialize(&mut parser).is_err());
        assert_eq!(parser.scratch_capacity(), 0);
    }
}

#[cfg(feature = "raw_value")]
#[test]
fn raw_and_number_protocols_stream_without_materialization() {
    let value = serde_json::value::RawValue::from_string("{\"x\": [1, 2]}".into()).unwrap();
    let mut output = Vec::new();
    native::to_writer(&mut output, &value).unwrap();
    assert_eq!(output, serde_json::to_vec(&value).unwrap());
    let number: serde_json::Number = serde_json::from_str("12345678901234567890").unwrap();
    output.clear();
    native::to_writer(&mut output, &number).unwrap();
    assert_eq!(output, serde_json::to_vec(&number).unwrap());
}

struct PanicValue;
impl Serialize for PanicValue {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        panic!("serializer panic");
    }
}
struct PanicSequence;
impl Serialize for PanicSequence {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut sequence = s.serialize_seq(Some(2))?;
        sequence.serialize_element(&42)?;
        sequence.serialize_element(&PanicValue)?;
        sequence.end()
    }
}
#[test]
fn reusable_and_streaming_serializers_recover_ownership_after_unwind() {
    let mut serializer = native::Serializer::with_capacity(2048);
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        serializer.serialize(&PanicSequence)
    }));
    assert!(panic.is_err());
    assert_eq!(serializer.buffer(), b"[42,");
    assert_eq!(serializer.capacity(), 2048);
    serializer.clear();
    serializer.serialize(&42).unwrap();
    assert_eq!(serializer.into_inner(), b"42");

    let mut serializer = native::Serializer::from_writer(Vec::new());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        serializer.serialize(&PanicSequence)
    }));
    assert!(panic.is_err());
    assert_eq!(serializer.into_inner().into_inner(), b"[42,");
}

struct DynamicField<'a> {
    key: &'static str,
    calls: &'a Cell<usize>,
}
impl Serialize for DynamicField<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut fields = serializer.serialize_struct("DynamicField", 1)?;
        fields.serialize_field(
            self.key,
            &Tracked {
                calls: self.calls,
                fail: false,
            },
        )?;
        fields.end()
    }
}
#[test]
fn short_key_batching_validates_names_and_preserves_error_order() {
    for key in [
        "",
        "a",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "é",
        "🦀🦀🦀🦀🦀🦀🦀🦀",
        "🦀🦀🦀🦀🦀🦀🦀🦀a",
        "quote\"",
        "slash\\",
        "control\n\u{1f}",
    ] {
        let calls = Cell::new(0);
        let expected = serde_json::to_vec(&DynamicField { key, calls: &calls }).unwrap();
        calls.set(0);
        assert_eq!(
            native::to_bytes(&DynamicField { key, calls: &calls }).unwrap(),
            expected
        );
        for budget in 0..expected.len() + 1 {
            let a = Cell::new(0);
            let b = Cell::new(0);
            let mut ours = BudgetWriter {
                bytes: Vec::new(),
                budget,
                interrupted: true,
            };
            let mut oracle = BudgetWriter {
                bytes: Vec::new(),
                budget,
                interrupted: true,
            };
            let candidate = native::to_writer(&mut ours, &DynamicField { key, calls: &a });
            let reference = serde_json::to_writer(&mut oracle, &DynamicField { key, calls: &b });
            assert_eq!(candidate.is_ok(), reference.is_ok());
            assert_eq!(ours.bytes, oracle.bytes, "key {key:?}, budget {budget}");
            assert_eq!(a.get(), b.get(), "key {key:?}, budget {budget}");
        }
    }
}

#[test]
fn native_128_bit_endpoints_use_fallible_sinks() {
    for value in [i128::MIN, i128::MAX, -1, 0, 1] {
        let expected = serde_json::to_vec(&value).unwrap();
        assert_eq!(native::to_bytes(&value).unwrap(), expected);
        let mut output = Vec::new();
        native::to_writer(&mut output, &value).unwrap();
        assert_eq!(output, expected);
        assert_eq!(native::from_slice::<i128>(&output).unwrap(), value);
    }
    for value in [u128::MAX, u64::MAX as u128 + 1, 0, 1] {
        let expected = serde_json::to_vec(&value).unwrap();
        assert_eq!(native::to_bytes(&value).unwrap(), expected);
        assert_eq!(native::from_slice::<u128>(&expected).unwrap(), value);
        for budget in 0..expected.len() + 1 {
            let mut ours = BudgetWriter {
                bytes: Vec::new(),
                budget,
                interrupted: true,
            };
            let mut oracle = BudgetWriter {
                bytes: Vec::new(),
                budget,
                interrupted: true,
            };
            assert_eq!(
                native::to_writer(&mut ours, &value).is_ok(),
                serde_json::to_writer(&mut oracle, &value).is_ok()
            );
            assert_eq!(ours.bytes, oracle.bytes);
        }
    }
}
