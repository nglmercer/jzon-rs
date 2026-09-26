# jzon-rs

[![Crates.io](https://img.shields.io/crates/v/jzon-rs.svg)](https://crates.io/crates/jzon-rs)
[![Docs.rs](https://docs.rs/jzon-rs/badge.svg)](https://docs.rs/jzon-rs)
[![CI](https://github.com/Rajaniraiyn/jzon-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Rajaniraiyn/jzon-rs/actions)
[![MSRV](https://img.shields.io/badge/rustc-1.71%2B-blue.svg)](https://blog.rust-lang.org/2022/11/03/Rust-1.71.0.html)

Zero-copy JSON for Rust. A proc-macro generates a typed, monomorphised
parser and serializer per struct at compile time — no runtime dispatch,
no intermediate `Value`, no unnecessary allocations.

## Three modes

### Mode A — custom derive (fastest)

Add `jzon-rs`. The `derive` feature is on by default.

```toml
[dependencies]
jzon-rs = "0.3"
```

```rust
use jzon::{ToJson, FromJson};

#[derive(ToJson, FromJson)]
#[serde(rename_all = "camelCase")]
struct User<'a> {
    id:    u64,
    name:  &'a str,  // zero-copy: borrows directly from the input bytes
    score: f64,
}

let user = User::from_json_str(input)?;
let out  = user.to_json_string();
```

### Mode B — any serde type

Add `jzon-rs-serde`. No other changes to your code.

```toml
[dependencies]
jzon-rs-serde = "0.3"
serde = { version = "1", features = ["derive"] }
```

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User<'a> { id: u64, name: &'a str }

let user: User = jzon_serde::from_str(input)?;
let out = jzon_serde::to_string(&user)?;
```

### Mode C — drop-in for serde_json

One line per crate in `Cargo.toml`, via a dependency rename. Zero code changes
required — and because compat re-exports `serde_json`'s own types, renamed
crates interoperate seamlessly with third-party crates (reqwest, axum, etc.)
still on real `serde_json`.

```toml
[dependencies]
serde_json = { package = "jzon-rs-compat", version = "0.3" }
```

`serde_json` features map 1:1 to same-named `jzon-rs-compat` flags
(`arbitrary_precision`, `preserve_order`, `raw_value`, `float_roundtrip`,
`unbounded_depth`). Note: `[patch.crates-io]` cannot express this swap —
cargo silently ignores renamed patches, and this crate itself depends on real
`serde_json` (fallback + type re-exports). See the
[`jzon-rs-compat` README](crates/jzon_compat/README.md) for the verified
details.

## Features

### jzon-rs

| Feature | Default | What it does |
|---------|---------|-------------|
| `derive` | ✓ | `#[derive(ToJson, FromJson)]` proc-macros |
| `serde` | | `jzon::from_str` / `to_string` for any serde type (Mode B engine) |
| `compat` | | `jzon::compat` — `serde_json`-compatible API (engine + fallback) |
| `simd` | | u128 SWAR (16 bytes/iter) |
| `simd-intrinsics` | | Hand-written `std::arch` kernels — aarch64 NEON, x86_64 SSE2/AVX2 |
| `simd + unstable` | | `std::simd` portable SIMD, 32–64 bytes/iter (nightly) |
| `fast-float` | | no-op (exact float backends are always on) |
| `zmij-float-ser` | | [zmij](https://crates.io/crates/zmij) (Schubfach+yy) float ser instead of ryu. ~30 % faster on Linux, ~10 % slower on Apple Silicon. MSRV 1.71. |
| `stats` | | per-parse allocation counters on Scanner |
| `strict` | | core `parse` rejects trailing commas (serde paths always do) |
| `unbounded_depth` | | lift the 128-level recursion limit |
| `arbitrary_precision` | | exact big numbers + forward to `serde_json` |
| `preserve_order` | | forward to `serde_json` (insertion-ordered `Map`) |
| `raw_value` | | forward to `serde_json` (`RawValue` API) |
| `float_roundtrip` | | forward to `serde_json` (exact fallback parsing) |

### jzon-rs-serde / jzon-rs-compat

Both crates forward the perf flags: `simd`, `fast-float`, `unstable`,
`stats`. `jzon-rs-compat` has `fast-float` **on by default** (sensible for a
drop-in replacement). `jzon-rs-serde` additionally forwards the
engine-affecting mirrors `arbitrary_precision` and `unbounded_depth`;
`jzon-rs-compat` mirrors the full `serde_json` set (`arbitrary_precision`,
`preserve_order`, `raw_value`, `float_roundtrip`, `unbounded_depth`, plus
`std`/`alloc` no-ops) so renames stay total — enable these instead of
`serde_json`'s flags directly.

## Benchmarks

<!-- bench:speedups-start -->
Up to **3.9× serde_json**, **2.4× sonic-rs**, **3.7× simd-json** on real-world workloads.
<!-- bench:speedups-end -->
<!-- bench:top-ser-start -->
Top: **57.70 GiB/s** twitter serialize
<!-- bench:top-ser-end -->.

<!-- bench:headline-start -->
| Platform | twitter de | twitter ser | citm de | canada ser |
|---|--:|--:|--:|--:|
| x86_64 Linux (AVX2)          | 1.58 GiB/s | 57.70 GiB/s | 2.57 GiB/s |  734 MiB/s |
| aarch64 Linux (Graviton)     | 1.27 GiB/s | 39.5 GiB/s | 2.36 GiB/s |  916 MiB/s |
| Apple Silicon (macOS)        | 1.35 GiB/s | 55.92 GiB/s | 2.66 GiB/s |  901 MiB/s |
| x86_64 Windows (AVX2)        | 1.35 GiB/s | 43.72 GiB/s | 2.16 GiB/s |  492 MiB/s |
| Windows on ARM               | 1.15 GiB/s | 38.58 GiB/s | 2.33 GiB/s |  642 MiB/s |
| **Best across platforms** | 1.58 GiB/s | 57.70 GiB/s | 2.66 GiB/s |  916 MiB/s |
<!-- bench:headline-end -->

Full matrix + competitor comparison + workloads where we lose:
[`BENCHMARKS.md`](./BENCHMARKS.md).

## How it works

- **Field dispatch as `u64` compare** — keys ≤ 8 bytes match in one
  CPU instruction. A one-word field-hint variable predicts the next
  key, so in-order JSON dispatches O(1) without hashing.
- **Zero-copy** — `&'de str` fields borrow directly from input bytes;
  no allocation unless the string has escapes.
- **Hand-written SIMD** — aarch64 NEON + x86_64 SSE2/AVX2 intrinsics
  for `find_quote_or_backslash` and `find_escape`. Up to 5.6× over u128 SWAR.
- **`fast_float2`** for parsing, **`ryu`** or **`zmij`** for serializing.

## Serde attributes supported

`rename`, `rename_all` (8 modes), `skip`, `skip_serializing`,
`skip_deserializing`, `skip_serializing_if`, `default`, `alias`,
`deny_unknown_fields`, `tag` (internally-tagged enums), `transparent`.

Types: all primitives, `String`, `&'de str`, `Option<T>`, `Vec<T>`,
`HashMap`, `BTreeMap`, `char`, `()`, tuples 1–12, `u128`/`i128`,
newtype structs, tuple structs, enum struct variants.

---

Made with ❤️ by [Rajaniraiyn](https://github.com/rajaniraiyn)
