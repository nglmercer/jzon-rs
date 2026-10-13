# jzon-rs

[![crates.io](https://img.shields.io/crates/v/jzon-rs.svg)](https://crates.io/crates/jzon-rs)
[![docs.rs](https://docs.rs/jzon-rs/badge.svg)](https://docs.rs/jzon-rs)
[![CI](https://github.com/nglmercer/jzon-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nglmercer/jzon-rs/actions)
[![MSRV](https://img.shields.io/badge/rustc-1.71%2B-blue.svg)](https://blog.rust-lang.org/2022/11/03/Rust-1.71.0.html)

Zero-copy JSON for Rust with compile-time generated parsers.

## Quick Start

```toml
[dependencies]
jzon-rs = "0.4"
```

```rust
use jzon::{ToJson, FromJson};

#[derive(ToJson, FromJson)]
struct Event<'a> {
    id: u64,
    name: &'a str,   // zero-copy — points directly into the input buffer
    tags: Vec<&'a str>,
}

fn main() {
    let src = r#"{"id":1,"name":"launch","tags":["rust","json"]}"#;
    let ev: Event = Event::from_json_str(src).unwrap();
    println!("{}", ev.to_json_string());
}
```

## Optional features

| Feature | What it adds |
|---------|-------------|
| `derive` (default) | `#[derive(ToJson, FromJson)]` proc-macros |
| `serde` | `jzon::from_str` / `to_string` for any `serde`-deriving type |
| `compat` | `jzon::compat` — `serde_json`-compatible API (`Value`, `json!`, etc.) |
| `simd` | u128 SWAR scanning (16 bytes/iter) |
| `simd-intrinsics` | Hand-written `std::arch` kernels — aarch64 NEON, x86_64 SSE2/AVX2 |
| `fast-float` | `ryu` float serialization, `fast_float2` parsing |
| `zmij-float-ser` | Use [`zmij`](https://crates.io/crates/zmij) (Schubfach + yy_double) for float serialization instead of `ryu`. See "Float serialization backend" below for tradeoffs. MSRV 1.71. |
| `unstable` | `std::simd` portable SIMD 32–64 bytes/iter (nightly only) |
| `stats` | Allocation counters on `Scanner` |

### Float serialization backend

`zmij-float-ser` swaps `ryu` for [`zmij`](https://crates.io/crates/zmij). Wins ~30% on Linux, loses ~10% on Apple Silicon — see [#3](https://github.com/Rajaniraiyn/jzon-rs/pull/3#issuecomment-4709984480) for numbers.

### Using the serde feature

```toml
[dependencies]
jzon-rs = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
```

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User<'a> { id: u64, name: &'a str }

let user: User = jzon::from_str(src).unwrap();
let out = jzon::to_string(&user).unwrap();
```

### Using the compat feature

```toml
[dependencies]
jzon-rs = { version = "0.4", features = ["compat"] }
```

```rust
use jzon::compat as serde_json;  // hot-path via jzon, types from serde_json

let user: User = serde_json::from_str(src).unwrap();
let v: serde_json::Value = serde_json::from_str(src).unwrap();
```

## Highlights

- **Zero-copy** — `&'a str` fields borrow directly from the input; no heap allocation for string data.
- **SIMD scanning** — vectorised byte-search on x86-64 and aarch64 for structural character detection.
- **No `unsafe` in user code** — the derive macros emit fully safe Rust.
- **serde attribute compatibility** — `#[serde(rename = "…")]`, `#[serde(skip_serializing_if)]`, etc. are honoured by the derive macros.

## Performance

<!-- bench:speedups-start -->
Performance claims from the earlier benchmark pipeline are withdrawn. See [current methodology and measurements](../../BENCHMARKS.md).
<!-- bench:speedups-end -->
<!-- bench:top-ser-start -->
Current mode-specific results are reported with emitted bytes and time per operation
<!-- bench:top-ser-end -->.
Full matrix: [`BENCHMARKS.md`](../../BENCHMARKS.md).

## Other Crates

| Crate | Purpose |
|-------|---------|
| [`jzon-rs-serde`](https://crates.io/crates/jzon-rs-serde) | Standalone serde `Serializer`/`Deserializer` (included via `serde` feature) |
| [`jzon-rs-compat`](https://crates.io/crates/jzon-rs-compat) | Dependency rename replacing `serde_json` per crate (zero code changes) |

## License

MIT

---

Made with ❤️ by [Rajaniraiyn](https://github.com/rajaniraiyn)

## Migration scope and readiness

Read [replacement readiness](../../docs/replacement-readiness.md) before migration.
Mode C preserves upstream types, callbacks, error construction and streaming by
re-exporting upstream once; it retains the serde_json dependency and claims no
native acceleration. Mode B is an independent native parser/serializer with its
own errors; its reader buffers input, while `to_writer` streams output directly.
`to_writer_buffered` retains explicit complete-output buffering. Mode A implements
an explicit attribute subset and accepts some trailing commas without `strict`.
Unescaped strings can borrow; escaped strings allocate. Byte strings, ignored
values and RawValue intentionally have different Unicode validation rules.

DepthGuard now owns shared counter state and has no lifetime parameter. It is
safe to move/drop the scanner before the guard. Unwind restores the budget;
leaking a guard conservatively reduces it. The first composite parse lazily allocates counter state; scalar parsing
and unescaped string borrowing need no counter allocation. Native Serde uses a
private scoped borrow of the entire parser for depth restoration, avoiding
atomic counter operations; public Scanner guards retain independent ownership. Enabling `unbounded_depth` does not disable the default limit:
call the deserializer/scanner method explicitly and manage stack safety yourself.
Statistics fields are now `decoded_strings` and `number_bytes_scanned` to avoid
implying total allocation or total scan counts. These API/feature changes warrant
a minor version bump for this pre-1.0 project. No release has been published.

Native reusable output: `Serializer::with_capacity(n)`, `serialize(value)`,
`clear()` (retain capacity), and `into_inner()`. Calls append; errors/panics
retain partial bytes. Escaped-string scratch reuses up to 64 KiB and releases
larger buffers on return or unwind. See [optimization progress](../../docs/optimization-progress.md).

## Planned 0.4.0 migration and safe facade source

Native `from_reader` parses incrementally; `from_reader_buffered` retains the
former read-to-end contract. Reader streams own their decoded values, slice
streams can borrow, and `into_parts()` preserves reader lookahead. Native Serde
errors expose category/line/column; match underlying variants via `cause()`.
Disable defaults and enable `alloc,serde,derive` for no_std on targets with
pointer-width atomics. Std gates I/O and HashMap APIs. Native-only mirror flags do
not activate upstream JSON.

The core `compat` module and standalone strict facade require the audited
serde_json 1.0.151 source. Its registry version has a confirmed non-ASCII bool-key
safety defect. Consumers need a workspace-root override to the audited source:

```toml
[patch.crates-io]
serde_json = { path = "/absolute/path/to/jzon-rs/vendor/serde_json" }
```

Packaged manifests lose path dependencies; registry-only publication is blocked
until a verified safe registry source or an explicitly migrated maintained fork
is available. No fixed upstream release is assumed. Facades still delegate
upstream and require its dependency. The new incremental reader is slower on the
measured integer-array workload. See [migration](../../docs/migration-0.4.md) and
[replacement readiness](../../docs/replacement-readiness.md) for separate verdicts
and executed evidence.
