//! Affected upstream reproductions run only under Miri.
#![cfg(miri)]
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct Key(bool);
#[test]
fn both_facades_public_constructors_and_streams() {
    for input in [
        r#"{"é":1}"#,
        r#"{"😀":1}"#,
        r#"{"":1}"#,
        r#"{"t\u0072ue":1}"#,
        r#"{"truex":1}"#,
        r#"{"true":1,"é":2}"#,
        r#"{"é\n":1}"#,
        r#"{"\u00e9":1}"#,
    ] {
        assert!(jzon::compat::from_str::<BTreeMap<bool, u8>>(input).is_err());
        assert!(jzon_compat::from_str::<BTreeMap<Key, u8>>(input).is_err());
        let mut de = jzon::compat::Deserializer::from_str(input);
        assert!(BTreeMap::<Key, u8>::deserialize(&mut de).is_err());
        let mut de = jzon_compat::Deserializer::from_str(input);
        assert!(BTreeMap::<bool, u8>::deserialize(&mut de).is_err());
        assert!(de.into_iter::<BTreeMap<Key, u8>>().next().unwrap().is_err());
        let mut stream =
            jzon::compat::Deserializer::from_str(input).into_iter::<BTreeMap<bool, u8>>();
        assert!(stream.next().unwrap().is_err());
        assert!(jzon_compat::Deserializer::from_str(input)
            .into_iter::<BTreeMap<Key, u8>>()
            .next()
            .unwrap()
            .is_err());
    }
    for input in [r#"{"true":1,"false":2}"#, r#"{}"#] {
        assert_eq!(
            jzon::compat::from_str::<BTreeMap<bool, u8>>(input).unwrap(),
            jzon_compat::from_str::<BTreeMap<bool, u8>>(input).unwrap()
        );
    }
}
