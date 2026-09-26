//! Arbitrary-precision parity: with `jzon/arbitrary_precision` (which also
//! enables serde_json's flag on the dependency), out-of-range integers and
//! all floats survive verbatim through the engine, exactly like native
//! serde_json.
//!
//! Run with: cargo test -p jzon-rs --features serde,arbitrary_precision --test arb_precision

use jzon_serde::{from_str, to_string};
use serde_json::{Number, Value};

fn num_str(v: &Value) -> &str {
    match v {
        Value::Number(n) => n.as_str(),
        other => panic!("expected Number, got {other:?}"),
    }
}

#[test]
fn big_int_exact_in_value() {
    let digits = "9".repeat(500);
    let ours: Value = from_str(&digits).unwrap();
    let theirs: Value = serde_json::from_str(&digits).unwrap();
    assert_eq!(ours, theirs);
    assert_eq!(num_str(&ours), digits);
}

#[test]
fn big_int_exact_in_number() {
    let digits = format!("-{}", "7".repeat(100));
    let ours: Number = from_str(&digits).unwrap();
    assert_eq!(ours.as_str(), digits);
    // Serializing an arbitrary-precision Number goes through the
    // `$serde_json::private::Number` token struct: digits emitted verbatim.
    assert_eq!(to_string(&ours).unwrap(), digits);
    assert_eq!(
        to_string(&ours).unwrap(),
        serde_json::to_string(&ours).unwrap()
    );
}

#[test]
fn u64_max_plus_one_exact() {
    let ours: Value = from_str("18446744073709551616").unwrap();
    assert_eq!(num_str(&ours), "18446744073709551616");
    assert_eq!(
        ours,
        serde_json::from_str::<Value>("18446744073709551616").unwrap()
    );
}

#[test]
fn ordinary_float_exact() {
    // Under arbitrary_precision serde_json preserves even fitting floats.
    // (Exponent forms are normalized by `Number::from_str` — `1e10` becomes
    // `1e+10` — identically on both sides.)
    for f in ["1.5", "-0.0", "3.14159", "1e10", "2.5E-7"] {
        let ours: Value = from_str(f).unwrap();
        let theirs: Value = serde_json::from_str(f).unwrap();
        assert_eq!(ours, theirs, "input: {f}");
        assert_eq!(num_str(&ours), num_str(&theirs), "input: {f}");
    }
    assert_eq!(num_str(&from_str::<Value>("1e10").unwrap()), "1e+10");
    assert_eq!(num_str(&from_str::<Value>("2.5E-7").unwrap()), "2.5e-7");
}

#[test]
fn float_overflow_exact() {
    let ours: Value = from_str("1e999").unwrap();
    let theirs: Value = serde_json::from_str("1e999").unwrap();
    assert_eq!(ours, theirs);
    assert_eq!(num_str(&ours), "1e+999");
}

#[test]
fn fitting_ints_still_typed() {
    let ours: Value = from_str("42").unwrap();
    assert_eq!(ours, Value::Number(Number::from(42u64)));
    let neg: Value = from_str("-42").unwrap();
    assert_eq!(neg, Value::Number(Number::from(-42i64)));
    // Typed scalar paths are unaffected by the flag.
    assert_eq!(from_str::<u64>("42").unwrap(), 42);
    assert_eq!(from_str::<i64>("-42").unwrap(), -42);
    assert_eq!(from_str::<f64>("1.5").unwrap(), 1.5);
    assert!(from_str::<u64>("18446744073709551616").is_err());
}

#[test]
fn nested_structures_exact() {
    let input = r#"{"big": 18446744073709551616, "pi": 3.14159, "arr": [1e999, -0.0]}"#;
    let ours: Value = from_str(input).unwrap();
    let theirs: Value = serde_json::from_str(input).unwrap();
    assert_eq!(ours, theirs);
    assert_eq!(
        to_string(&ours).unwrap(),
        serde_json::to_string(&theirs).unwrap()
    );
}
