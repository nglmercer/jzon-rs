# jzon-rs-serde

[![crates.io](https://img.shields.io/crates/v/jzon-rs-serde.svg)](https://crates.io/crates/jzon-rs-serde)
[![docs.rs](https://docs.rs/jzon-rs-serde/badge.svg)](https://docs.rs/jzon-rs-serde)
[![MSRV](https://img.shields.io/badge/rustc-1.71%2B-blue.svg)](https://blog.rust-lang.org/2022/11/03/Rust-1.71.0.html)

SIMD-backed serde `Serializer`/`Deserializer` for any type deriving `serde::Serialize`/`serde::Deserialize`.

## Usage

```toml
[dependencies]
jzon-rs-serde = "0.4"
serde = { version = "1", features = ["derive"] }
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User<'a> {
    id: u64,
    name: &'a str,   // zero-copy via visit_borrowed_str
}

fn main() {
    let src = r#"{"id":42,"name":"ada"}"#;
    let user: User = jzon_serde::from_str(src).unwrap();
    let out: String = jzon_serde::to_string(&user).unwrap();
    println!("{out}");
}
```

Zero-copy `&str` fields work transparently: the deserializer calls `visit_borrowed_str`, so the string data is borrowed directly from the input slice with no allocation.

## Feature Flags

Feature flags mirror those of [jzon-rs](https://crates.io/crates/jzon-rs).

| Flag | Default | Description |
|------|---------|-------------|
| `simd` | off | u128 SWAR (16 bytes/iter) scanning |
| `fast-float` | off | Compatibility no-op; use `float_roundtrip` for the named parsing policy |
| `unstable` | off | `std::simd` portable SIMD 32–64 bytes/iter (nightly only) |
| `stats` | off | Scanner decoded-string and number-byte counters; not total allocator activity |

## Part of the jzon family

| Crate | Purpose |
|-------|---------|
| [jzon-rs](https://crates.io/crates/jzon-rs) | Core zero-copy JSON with `#[derive(ToJson, FromJson)]` |
| [jzon-rs-compat](https://crates.io/crates/jzon-rs-compat) | Drop-in `serde_json` replacement |

## License

MIT

## Native contract and migration

This wrapper exposes the independent native engine with default std and optional no_std + alloc. It is separate from the delegated compatibility facade. Native errors retain a distinct type and messages, with category, line and column. `from_reader` parses incrementally; `ReaderDeserializer` supports fused streams and preserves one lookahead byte through `into_parts`. `from_reader_buffered` retains read-to-end behavior explicitly. `to_writer` now streams directly, propagates I/O failures immediately, and preserves partial output. `to_writer_buffered` retains the earlier buffer-before-I/O behavior. Mode C retains upstream error types and reader semantics.

`to_string`, `to_bytes`, `to_writer` and `to_bytes_in` accept `Serialize + ?Sized`. `to_bytes_in(value, &mut buffer)` appends directly with no intermediate copy; clear the buffer for reset semantics. Failure retains partial output, including during panic unwinding.

Default native float parsing reproduces the pinned reference's significand/scaling policy; `float_roundtrip` selects correctly rounded fast-float2 parsing and direct f32 rounding. Serialization uses zmij, matching serde_json 1.0.151's formatter. Mode A has its own policies. Non-finite values serialize to null; native Serde preserves signed zero.

`unbounded_depth` keeps the default limit. Construct `Deserializer::from_str`/`from_slice`, call `disable_recursion_limit()` explicitly, deserialize once, then call `end()`. Deep parsing and destruction require adequate stack management.

Native options include `simd`, `simd-intrinsics`, nightly `unstable`, `stats`, `raw_value`, `arbitrary_precision`, and `float_roundtrip`. Disable defaults and enable `alloc` for no_std. Derive-generated core/alloc paths and native Serde are compiled on thumbv7em-none-eabi. HashMap and I/O APIs require `std`; guard counters require target atomics. Native-only mirror features do not activate serde_json. See [readiness](../../docs/replacement-readiness.md) for executed evidence and remaining compatibility gates.

Reusable native serializers use `Serializer::new()`, `with_capacity(n)`, or
`from_vec(buffer)`; `serialize(&value)` appends, `clear()` resets length while
retaining capacity, and `into_inner()` returns the buffer. User failures and
panics retain emitted bytes. `Serializer::from_writer(writer)` writes directly;
`into_inner().into_inner()` recovers that writer.

Native decoding preserves `visit_borrowed_str` for plain input and `visit_str`
for escaped input. Transient decoding scratch is reused, with retained capacity
bounded at 64 KiB after return/error/unwind. `Deserializer::clear_scratch()`
releases it explicitly. The native depth guard exclusively borrows the whole
parser and restores its local budget on success/error/unwind without atomics.
