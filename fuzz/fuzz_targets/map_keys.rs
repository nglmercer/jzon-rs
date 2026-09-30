#![no_main]
#[path = "../../crates/jzon/tests/support/map_keys.rs"]
mod map_keys;
use libfuzzer_sys::fuzz_target;
use map_keys::{ByteKey, EnumKey, NewtypeKey, UnitKey};
use serde::de::DeserializeOwned;
use std::{collections::BTreeMap, fmt::Debug};

fn compare<T: DeserializeOwned + PartialEq + Debug>(data: &[u8]) {
    let a = jzon::serde_impl::from_slice::<T>(data);
    let b = reference_json::from_slice::<T>(data);
    assert_eq!(a.is_ok(), b.is_ok(), "slice {data:?}: {a:?} vs {b:?}");
    if let (Ok(a), Ok(b)) = (a, b) {
        assert_eq!(a, b);
    }
    let reader = jzon::serde_impl::from_reader::<_, T>(data);
    let reference = reference_json::from_reader::<_, T>(data);
    assert_eq!(
        reader.is_ok(),
        reference.is_ok(),
        "reader {data:?}: {reader:?} vs {reference:?}"
    );
    if let (Ok(a), Ok(b)) = (reader, reference) {
        assert_eq!(a, b);
    }
    if let Ok(text) = std::str::from_utf8(data) {
        let a = jzon::serde_impl::from_str::<T>(text);
        let b = reference_json::from_str::<T>(text);
        assert_eq!(a.is_ok(), b.is_ok(), "str {text:?}: {a:?} vs {b:?}");
        if let (Ok(a), Ok(b)) = (a, b) {
            assert_eq!(a, b);
        }
    }
}
fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    compare::<BTreeMap<(), u8>>(data);
    compare::<BTreeMap<UnitKey<false>, u8>>(data);
    compare::<BTreeMap<UnitKey<true>, u8>>(data);
    compare::<BTreeMap<EnumKey, u8>>(data);
    compare::<BTreeMap<bool, u8>>(data);
    compare::<BTreeMap<NewtypeKey, u8>>(data);
    compare::<BTreeMap<ByteKey<false>, u8>>(data);
    compare::<BTreeMap<ByteKey<true>, u8>>(data);
});
