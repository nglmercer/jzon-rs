#![no_main]
#[path = "../../crates/jzon/tests/support/map_keys.rs"]
mod map_keys;
use libfuzzer_sys::fuzz_target;
use map_keys::{ByteKey, EnumKey, NewtypeKey, UnitKey};
use serde::de::DeserializeOwned;
use std::{collections::BTreeMap, fmt::Debug};

fn compare<T: DeserializeOwned + PartialEq + Debug>(data: &[u8], boolean_keys: bool) {
    let a = jzon::serde_impl::from_slice::<T>(data);
    let b = reference_json::from_slice::<T>(data);
    assert_eq!(a.is_ok(), b.is_ok(), "slice {data:?}: {a:?} vs {b:?}");
    if let (Ok(a), Ok(b)) = (a, b) {
        assert_eq!(a, b);
    }
    if let Ok(text) = std::str::from_utf8(data) {
        let a = jzon::serde_impl::from_str::<T>(text);
        // Pinned serde_json 1.0.151 StrRead drops the first byte of an
        // unknown bool key before constructing its string error. Non-ASCII
        // input can violate UTF-8 invariants (isolated Miri evidence in docs).
        // Keep testing BOTH native APIs on every input; use the pinned slice
        // oracle for this family/input domain rather than skipping cases.
        let b = if boolean_keys && !text.is_ascii() {
            reference_json::from_slice::<T>(data)
        } else {
            reference_json::from_str::<T>(text)
        };
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
    compare::<BTreeMap<(), u8>>(data, false);
    compare::<BTreeMap<UnitKey<false>, u8>>(data, false);
    compare::<BTreeMap<UnitKey<true>, u8>>(data, false);
    compare::<BTreeMap<EnumKey, u8>>(data, false);
    compare::<BTreeMap<bool, u8>>(data, true);
    compare::<BTreeMap<NewtypeKey, u8>>(data, true);
    compare::<BTreeMap<ByteKey<false>, u8>>(data, false);
    compare::<BTreeMap<ByteKey<true>, u8>>(data, false);
});
