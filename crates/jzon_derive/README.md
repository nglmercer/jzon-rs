# jzon-rs-derive

[![crates.io](https://img.shields.io/crates/v/jzon-rs-derive.svg)](https://crates.io/crates/jzon-rs-derive)
[![docs.rs](https://docs.rs/jzon-rs-derive/badge.svg)](https://docs.rs/jzon-rs-derive)
[![MSRV](https://img.shields.io/badge/rustc-1.71%2B-blue.svg)](https://blog.rust-lang.org/2022/11/03/Rust-1.71.0.html)

Proc-macro crate for `#[derive(ToJson, FromJson)]` — part of [jzon](https://crates.io/crates/jzon-rs).

> **Do not add this crate directly.**
> Add `jzon-rs` instead — it re-exports both macros automatically.

## Example

```rust
use jzon::{ToJson, FromJson};

#[derive(ToJson, FromJson)]
struct Point {
    x: f64,
    y: f64,
}
```

## See Also

- [jzon-rs](https://crates.io/crates/jzon-rs) — main crate and full documentation
- [GitHub](https://github.com/Rajaniraiyn/jzon-rs)

## License

MIT

## Supported attribute subset

Mode A supports rename, rename_all, aliases, skip/skip_serializing/skip_deserializing, skip_serializing_if, default, flatten, deny_unknown_fields, transparent, tag/content, untagged, other, and automatic borrowed string fields. `rjson(trie_dispatch)` and native rjson serialize_with/deserialize_with hooks are extensions. `borrow` is accepted as metadata; plain unescaped &str fields already borrow without it.

Serde custom hook attributes, remote/from/try_from/into, custom bound/expecting metadata, rename_all_fields and other unsupported attributes fail compilation. These derives do not implement every Serde semantic. Enum and flatten combinations need independent validation; consult [readiness](../../docs/replacement-readiness.md). Without strict, Mode A retains its documented trailing-comma leniency, distinct from native Serde's unconditional container syntax validation.
