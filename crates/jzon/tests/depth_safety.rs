use jzon::Scanner;

#[test]
fn guard_owns_state_after_owner_drops() {
    let mut scanner = Box::new(Scanner::new(b"null"));
    let guard = scanner.enter_depth().unwrap();
    drop(scanner);
    drop(guard); // Safe independent state ownership, including under Miri.
}

#[test]
fn restores_on_error_and_unwind() {
    let mut scanner = Scanner::new(b"null");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = scanner.enter_depth().unwrap();
        panic!("visitor panic");
    }));
    assert!(result.is_err());
    for _ in 0..256 {
        let _guard = scanner.enter_depth().unwrap();
    }
    let mut guards = Vec::new();
    for _ in 0..127 {
        guards.push(scanner.enter_depth().unwrap());
    }
    assert!(scanner.enter_depth().is_err());
    drop(guards);
    assert!(scanner.enter_depth().is_ok());
}

#[test]
fn public_string_variant_cannot_bypass_escaping() {
    use jzon::ToJson;
    assert_eq!(
        jzon::JsonStr::BorrowedNoEsc("\"\n").to_json_bytes(),
        b"\"\\\"\\n\""
    );
}

#[cfg(feature = "serde")]
#[test]
fn native_deserializer_restores_budget_after_visitor_unwind() {
    use serde::de::{Deserializer as _, SeqAccess, Visitor};
    struct PanicAfterClose;
    impl<'de> Visitor<'de> for PanicAfterClose {
        type Value = ();
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an empty array")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
            assert!(seq.next_element::<serde::de::IgnoredAny>()?.is_none());
            panic!("visitor panics after consuming the container");
        }
    }
    // The input position is intentionally retained, not rolled back. After
    // consuming the first container, recovery can parse the next framed value.
    let input = format!("[] {}null{}", "[".repeat(127), "]".repeat(127));
    let mut parser = jzon::serde_impl::Deserializer::from_str(&input);
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (&mut parser).deserialize_seq(PanicAfterClose)
    }));
    assert!(caught.is_err());
    let _: serde_json::Value = serde::Deserialize::deserialize(&mut parser).unwrap();
    parser.end().unwrap();
}
