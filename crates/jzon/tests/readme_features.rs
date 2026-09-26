//! Regression tests for features promised by the READMEs.
//!
//! Root `README.md` promises: `BTreeMap` values, `enum struct variants`,
//! `skip_serializing_if`, `alias`, `deny_unknown_fields`, `tag` and
//! `transparent` support in the derive macros. `crates/jzon/README.md`
//! additionally promises `serde` (`jzon::from_str` / `jzon::to_string`) and
//! `compat` (`jzon::compat`) cargo features on the `jzon` crate itself.

use jzon::{FromJson, ToJson};

// ── BTreeMap with borrowed keys ──────────────────────────────────────────────
// README: "Types: ... HashMap, BTreeMap, ..."

#[test]
fn btreemap_borrowed_keys_roundtrip() {
    use std::collections::BTreeMap;
    let json = r#"{"a":1,"b":2}"#;
    let m: BTreeMap<&str, u64> = FromJson::from_json_str(json).unwrap();
    assert_eq!(m.get("a").copied(), Some(1));
    assert_eq!(m.get("b").copied(), Some(2));
    assert_eq!(m.len(), 2);
}

#[test]
fn btreemap_borrowed_keys_are_zero_copy() {
    use std::collections::BTreeMap;
    let json = r#"{"alpha":1}"#;
    let m: BTreeMap<&str, u64> = FromJson::from_json_str(json).unwrap();
    let key: &str = *m.keys().next().unwrap();
    assert_eq!(key, "alpha");
    let key_ptr = key.as_ptr() as usize;
    let base = json.as_ptr() as usize;
    assert!(
        key_ptr >= base && key_ptr < base + json.len(),
        "borrowed BTreeMap key must point into the input"
    );
}

// ── Externally-tagged enums (serde default) ───────────────────────────────────
// README: "Types: ... enum struct variants."

#[derive(ToJson, FromJson, Debug, PartialEq)]
enum Message {
    Ping,
    Move { x: i32, y: i32 },
}

#[test]
fn external_enum_unit_variant_shape() {
    assert_eq!(Message::Ping.to_json_string(), r#""Ping""#);
    assert_eq!(Message::from_json_str(r#""Ping""#).unwrap(), Message::Ping);
}

#[test]
fn external_enum_unit_variant_map_form() {
    // serde also accepts the single-entry map form with null payload.
    assert_eq!(
        Message::from_json_str(r#"{"Ping":null}"#).unwrap(),
        Message::Ping
    );
}

#[test]
fn external_enum_struct_variant_roundtrip() {
    let v = Message::Move { x: -1, y: 2 };
    let json = v.to_json_string();
    assert_eq!(json, r#"{"Move":{"x":-1,"y":2}}"#);
    assert_eq!(Message::from_json_str(&json).unwrap(), v);
}

#[test]
fn external_enum_matches_serde_json() {
    #[derive(
        jzon::ToJson, jzon::FromJson, serde::Serialize, serde::Deserialize, Debug, PartialEq,
    )]
    enum Wire {
        Ping,
        Move { x: i32, y: i32 },
    }
    for v in [Wire::Ping, Wire::Move { x: 3, y: -4 }] {
        let jzon_out = v.to_json_string();
        let serde_out = serde_json::to_string(&v).unwrap();
        assert_eq!(jzon_out, serde_out);
        assert_eq!(Wire::from_json_str(&serde_out).unwrap(), v);
    }
}

#[test]
fn external_enum_unknown_variant_errors() {
    assert!(matches!(
        Message::from_json_str(r#""Pong""#),
        Err(jzon::Error::UnknownVariant)
    ));
    assert!(matches!(
        Message::from_json_str(r#"{"Pong":{}}"#),
        Err(jzon::Error::UnknownVariant)
    ));
}

#[derive(ToJson, FromJson, Debug, PartialEq)]
enum WithOther {
    Known,
    #[serde(other)]
    Fallback,
}

#[test]
fn external_enum_other_catches_unknown() {
    assert_eq!(
        WithOther::from_json_str(r#""Known""#).unwrap(),
        WithOther::Known
    );
    assert_eq!(
        WithOther::from_json_str(r#""Whatever""#).unwrap(),
        WithOther::Fallback
    );
    assert_eq!(
        WithOther::from_json_str(r#"{"Whatever":null}"#).unwrap(),
        WithOther::Fallback
    );
}

#[derive(ToJson, FromJson, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
enum StrictExternal {
    Point { x: u32 },
}

#[test]
fn external_enum_struct_variant_deny_unknown_fields() {
    let ok = StrictExternal::from_json_str(r#"{"Point":{"x":1}}"#).unwrap();
    assert_eq!(ok, StrictExternal::Point { x: 1 });
    assert!(matches!(
        StrictExternal::from_json_str(r#"{"Point":{"x":1,"zzz":2}}"#),
        Err(jzon::Error::UnknownField)
    ));
}

#[derive(ToJson, FromJson, Debug, PartialEq)]
enum AliasedExternal {
    Sized {
        #[serde(alias = "w")]
        width: u32,
    },
}

#[test]
fn external_enum_struct_variant_alias() {
    assert_eq!(
        AliasedExternal::from_json_str(r#"{"Sized":{"width":3}}"#).unwrap(),
        AliasedExternal::Sized { width: 3 }
    );
    assert_eq!(
        AliasedExternal::from_json_str(r#"{"Sized":{"w":4}}"#).unwrap(),
        AliasedExternal::Sized { width: 4 }
    );
}

// ── Internally-tagged enums: unit + empty variants ───────────────────────────
// README: "tag (internally-tagged enums)".

#[derive(ToJson, FromJson, Debug, PartialEq)]
#[serde(tag = "type")]
enum Tagged {
    Point,
    Circle { radius: f64 },
    Empty {},
}

#[test]
fn tagged_enum_unit_variant_includes_tag() {
    // serde serializes unit variants of internally-tagged enums as {"tag":"V"}.
    assert_eq!(Tagged::Point.to_json_string(), r#"{"type":"Point"}"#);
    assert_eq!(
        Tagged::from_json_str(r#"{"type":"Point"}"#).unwrap(),
        Tagged::Point
    );
}

#[test]
fn tagged_enum_unit_variant_matches_serde_json() {
    #[derive(
        jzon::ToJson, jzon::FromJson, serde::Serialize, serde::Deserialize, Debug, PartialEq,
    )]
    #[serde(tag = "type")]
    enum Shape {
        Point,
        Circle { radius: f64 },
    }
    for v in [Shape::Point, Shape::Circle { radius: 1.25 }] {
        let jzon_out = v.to_json_string();
        let serde_out = serde_json::to_string(&v).unwrap();
        assert_eq!(jzon_out, serde_out, "tagged output must match serde_json");
        assert_eq!(Shape::from_json_str(&serde_out).unwrap(), v);
    }
}

#[test]
fn tagged_enum_empty_struct_variant_has_no_trailing_comma() {
    assert_eq!(Tagged::Empty {}.to_json_string(), r#"{"type":"Empty"}"#);
    assert_eq!(
        Tagged::from_json_str(r#"{"type":"Empty"}"#).unwrap(),
        Tagged::Empty {}
    );
}

// ── skip_serializing_if inside enum struct variants ──────────────────────────
// README: "skip_serializing_if".

#[derive(ToJson, FromJson, Debug, PartialEq)]
enum MaybeLarge {
    Rect {
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        width: u32,
    },
}

#[test]
fn external_enum_variant_skip_serializing_if() {
    let v = MaybeLarge::Rect {
        label: None,
        width: 5,
    };
    assert_eq!(v.to_json_string(), r#"{"Rect":{"width":5}}"#);
    assert_eq!(MaybeLarge::from_json_str(&v.to_json_string()).unwrap(), v);

    let v = MaybeLarge::Rect {
        label: Some("w".into()),
        width: 5,
    };
    assert_eq!(v.to_json_string(), r#"{"Rect":{"label":"w","width":5}}"#);
    assert_eq!(MaybeLarge::from_json_str(&v.to_json_string()).unwrap(), v);
}

#[derive(ToJson, FromJson, Debug, PartialEq)]
#[serde(tag = "kind")]
enum MaybeTagged {
    Rect {
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        width: u32,
    },
}

#[test]
fn tagged_enum_variant_skip_serializing_if() {
    let v = MaybeTagged::Rect {
        label: None,
        width: 5,
    };
    assert_eq!(v.to_json_string(), r#"{"kind":"Rect","width":5}"#);
    assert_eq!(MaybeTagged::from_json_str(&v.to_json_string()).unwrap(), v);

    let v = MaybeTagged::Rect {
        label: Some("w".into()),
        width: 5,
    };
    assert_eq!(
        v.to_json_string(),
        r#"{"kind":"Rect","label":"w","width":5}"#
    );
    assert_eq!(MaybeTagged::from_json_str(&v.to_json_string()).unwrap(), v);
}

// ── alias + deny_unknown_fields inside tagged variants ───────────────────────
// README: "alias", "deny_unknown_fields".

#[derive(ToJson, FromJson, Debug, PartialEq)]
#[serde(tag = "t")]
enum TaggedAlias {
    Sized {
        #[serde(alias = "w")]
        width: u32,
    },
}

#[test]
fn tagged_enum_variant_field_alias() {
    assert_eq!(
        TaggedAlias::from_json_str(r#"{"t":"Sized","width":3}"#).unwrap(),
        TaggedAlias::Sized { width: 3 }
    );
    assert_eq!(
        TaggedAlias::from_json_str(r#"{"t":"Sized","w":4}"#).unwrap(),
        TaggedAlias::Sized { width: 4 }
    );
}

#[derive(ToJson, FromJson, Debug, PartialEq)]
#[serde(tag = "t")]
enum TaggedVariantAlias {
    #[serde(alias = "Sq")]
    Square { side: u32 },
}

#[test]
fn tagged_enum_variant_name_alias() {
    assert_eq!(
        TaggedVariantAlias::from_json_str(r#"{"t":"Square","side":2}"#).unwrap(),
        TaggedVariantAlias::Square { side: 2 }
    );
    assert_eq!(
        TaggedVariantAlias::from_json_str(r#"{"t":"Sq","side":2}"#).unwrap(),
        TaggedVariantAlias::Square { side: 2 }
    );
}

#[derive(ToJson, FromJson, Debug, PartialEq)]
#[serde(tag = "t", deny_unknown_fields)]
enum TaggedStrict {
    Point { x: u32 },
}

#[test]
fn tagged_enum_variant_deny_unknown_fields() {
    let ok = TaggedStrict::from_json_str(r#"{"t":"Point","x":1}"#).unwrap();
    assert_eq!(ok, TaggedStrict::Point { x: 1 });
    assert!(matches!(
        TaggedStrict::from_json_str(r#"{"t":"Point","x":1,"zzz":2}"#),
        Err(jzon::Error::UnknownField)
    ));
}

// ── Top-level trailing content is rejected ───────────────────────────────────

#[derive(ToJson, FromJson, Debug, PartialEq)]
struct Solo {
    x: u64,
}

#[test]
fn top_level_trailing_garbage_rejected() {
    assert!(Solo::from_json_str(r#"{"x":1} proves"#).is_err());
    assert!(Solo::from_json_str(r#"{"x":1} {"x":2}"#).is_err());
    assert!(Solo::from_json_bytes(br#"{"x":1} "#).is_ok());
}

// ── stats: hint hit/miss counters ─────────────────────────────────────────────
// README: "stats | per-parse allocation counters on Scanner".

#[cfg(feature = "stats")]
#[test]
fn stats_hint_counters_track_dispatch() {
    // In-order keys hit the field-hint cache every time.
    let mut sc = jzon::Scanner::new_str(r#"{"x":1}"#);
    Solo::from_json_scanner(&mut sc).unwrap();
    assert_eq!(sc.stats.hint_hits, 1);
    assert_eq!(sc.stats.hint_misses, 0);
    assert_eq!(sc.stats.hint_hit_rate(), Some(1.0));

    // Fully reversed keys miss every time.
    #[derive(FromJson)]
    struct Multi {
        a: u32,
        b: u32,
        c: u32,
        d: u32,
    }
    let mut sc = jzon::Scanner::new_str(r#"{"d":4,"c":3,"b":2,"a":1}"#);
    Multi::from_json_scanner(&mut sc).unwrap();
    assert_eq!(sc.stats.hint_hits, 0);
    assert_eq!(sc.stats.hint_misses, 4);
}

// ── jzon `serde` feature: jzon::from_str / jzon::to_string ────────────────────
// crates/jzon/README.md: "serde | jzon::from_str / to_string ...".

#[cfg(feature = "serde")]
mod serde_feature {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct User<'a> {
        id: u64,
        name: &'a str,
    }

    #[test]
    fn jzon_from_str_to_string_zero_copy() {
        let src = r#"{"id":42,"name":"ada"}"#;
        let user: User = jzon::from_str(src).unwrap();
        assert_eq!(user.id, 42);
        assert_eq!(user.name, "ada");
        // Zero-copy: borrowed field points into the input.
        let ptr = user.name.as_ptr() as usize;
        let base = src.as_ptr() as usize;
        assert!(ptr >= base && ptr < base + src.len());
        let out = jzon::to_string(&user).unwrap();
        assert_eq!(out, src);
    }

    #[test]
    fn jzon_slice_bytes_writer_reader_roundtrip() {
        let user = User { id: 7, name: "bob" };
        let bytes = jzon::to_bytes(&user).unwrap();
        let back: User = jzon::from_slice(&bytes).unwrap();
        assert_eq!(back, user);

        let mut buf = Vec::new();
        jzon::to_writer(&mut buf, &user).unwrap();
        let back: User = jzon::from_slice(&buf).unwrap();
        assert_eq!(back, user);

        // from_reader buffers internally, so it only serves owned types.
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct OwnedUser {
            id: u64,
            name: String,
        }
        let owned_src = jzon::to_string(&OwnedUser {
            id: 7,
            name: "bob".into(),
        })
        .unwrap();
        let cursor = std::io::Cursor::new(owned_src.into_bytes());
        let owned: OwnedUser = jzon::from_reader(cursor).unwrap();
        assert_eq!(owned.id, 7);
    }

    #[test]
    fn jzon_f32_parses_directly() {
        let v: f32 = jzon::from_str("0.1").unwrap();
        assert_eq!(v, 0.1f32);
        let v: f32 = jzon::from_str("1e3").unwrap();
        assert_eq!(v, 1000.0f32);
    }

    #[cfg(feature = "stats")]
    #[test]
    fn jzon_serde_stats_reported() {
        let src = r#"{"id":1,"name":"ada"}"#;
        let (user, stats): (User, jzon::stats::ScannerStats) =
            jzon::from_str_with_stats(src).unwrap();
        assert_eq!(user.id, 1);
        // Two object keys plus the borrowed `name` value.
        assert_eq!(stats.zero_copy_borrows, 3);
        assert_eq!(stats.heap_allocations, 0);
    }
}

// ── jzon `compat` feature: jzon::compat ───────────────────────────────────────
// crates/jzon/README.md: "compat | jzon::compat — serde_json-compatible API".

#[cfg(feature = "compat")]
mod compat_feature {
    use jzon::compat as serde_json;

    #[test]
    fn jzon_compat_roundtrip() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct User {
            id: u64,
            name: String,
        }
        let u = User {
            id: 1,
            name: "ada".into(),
        };
        let json = serde_json::to_string(&u).unwrap();
        let back: User = serde_json::from_str(&json).unwrap();
        assert_eq!(back, u);
    }

    #[test]
    fn jzon_compat_value_and_macro() {
        let v: serde_json::Value = serde_json::from_str(r#"{"n":42}"#).unwrap();
        assert_eq!(v["n"], serde_json::Value::Number(42.into()));
        let m = serde_json::json!({"hello": "world"});
        assert_eq!(m["hello"], serde_json::Value::String("world".into()));
    }
}
