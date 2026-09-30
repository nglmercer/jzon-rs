# jzon-rs

[![Crates.io](https://img.shields.io/crates/v/jzon-rs.svg)](https://crates.io/crates/jzon-rs)
[![Docs.rs](https://docs.rs/jzon-rs/badge.svg)](https://docs.rs/jzon-rs)
[![CI](https://github.com/nglmercer/jzon-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nglmercer/jzon-rs/actions)
[![MSRV](https://img.shields.io/badge/rustc-1.71%2B-blue.svg)](https://blog.rust-lang.org/2022/11/03/Rust-1.71.0.html)

Zero-copy JSON for Rust. A proc-macro generates a typed, monomorphised
parser and serializer per struct at compile time — no runtime dispatch,
no intermediate `Value`, no unnecessary allocations.

## Three modes

### Mode A — custom typed derives

Add `jzon-rs`. The `derive` feature is on by default.

```toml
[dependencies]
jzon-rs = "0.4"
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
jzon-rs-serde = "0.4"
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
serde_json = { package = "jzon-rs-compat", version = "0.4" }
```

`serde_json` features map 1:1 to same-named `jzon-rs-compat` flags
(`arbitrary_precision`, `preserve_order`, `raw_value`, `float_roundtrip`,
`unbounded_depth`). Note: `[patch.crates-io]` cannot express this swap —
cargo silently ignores renamed patches, and this crate itself depends on real
`serde_json` (single-invocation delegation + type re-exports). See the
[`jzon-rs-compat` README](crates/jzon_compat/README.md) for the verified
details.

## Features

### jzon-rs

| Feature | Default | What it does |
|---------|---------|-------------|
| `derive` | ✓ | `#[derive(ToJson, FromJson)]` proc-macros |
| `serde` | | `jzon::from_str` / `to_string` for any serde type (Mode B engine) |
| `compat` | | `jzon::compat` — `serde_json`-compatible API (upstream delegation) |
| `simd` | | u128 SWAR (16 bytes/iter) |
| `simd-intrinsics` | | Hand-written `std::arch` kernels — aarch64 NEON, x86_64 SSE2/AVX2 |
| `simd + unstable` | | `std::simd` portable SIMD, 32–64 bytes/iter (nightly) |
| `fast-float` | | no-op (exact float backends are always on) |
| `zmij-float-ser` | | [zmij](https://crates.io/crates/zmij) (Schubfach+yy) float ser instead of ryu. Mode A formatter option; Mode B always uses zmij to match the pinned reference. |
| `stats` | | scanner event counters; not allocator totals |
| `strict` | | core `parse` rejects trailing commas (serde paths always do) |
| `unbounded_depth` | | expose explicit `disable_recursion_limit`; default limit remains |
| `arbitrary_precision` | | exact big numbers + forward to `serde_json` |
| `preserve_order` | | forward to `serde_json` (insertion-ordered `Map`) |
| `raw_value` | | forward to `serde_json` (`RawValue` API) |
| `float_roundtrip` | | forward to `serde_json` (matching float_roundtrip policy) |

### jzon-rs-serde / jzon-rs-compat

`jzon-rs-serde` forwards native scanning options and the `arbitrary_precision`,
`raw_value`, `float_roundtrip`, and `unbounded_depth` policies. It requires std.
`jzon-rs-compat` delegates every operation to pinned upstream serde_json 1.0.151.
It defaults to `std`, supports genuine `no_std + alloc`, and forwards upstream
features. Its legacy performance flags are explicit no-ops. `jzon::compat` is
also delegated, but the core crate still requires std.

## Benchmarks

<!-- bench:speedups-start -->
Performance claims from the earlier benchmark pipeline are withdrawn. See [current methodology and measurements](BENCHMARKS.md).
<!-- bench:speedups-end -->
<!-- bench:top-ser-start -->
Current mode-specific results are reported with emitted bytes and time per operation
<!-- bench:top-ser-end -->.

<!-- bench:headline-start -->
Historical throughput tables are excluded from current readiness conclusions. See `docs/benchmark-history.md` for the archived, unvalidated figures.
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
  for `find_quote_or_backslash` and `find_escape`. Runtime CPU dispatch is tested separately from workload performance.
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

## Migration scope and readiness

Read [replacement readiness](docs/replacement-readiness.md) before migration.
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
larger buffers on return or unwind. See [optimization progress](./docs/optimization-progress.md).

## Planned 0.4.0 migration

Native `from_reader` is incremental. Use `from_reader_buffered` for the former
read-to-end contract. `ReaderDeserializer::new(reader).into_iter::<T>()` and
`Deserializer::from_str(input).into_iter::<T>()` parse consecutive native values
without retries. Reader callbacks receive transient strings/bytes; slice streams
can borrow. `into_parts` returns any prefetched byte with the reader.

Native errors expose category, line and column; match `error.cause()` when migrating
old enum-pattern checks. Disable default features and enable `alloc,serde,derive`
for embedded allocation support. I/O and HashMap APIs require `std`.

Both compatibility facades require the audited serde_json 1.0.151 source override
at the consuming workspace root. Registry-only publication is blocked because
Cargo packaging strips path dependencies. No newer safe release is assumed.
See [current implementation and evidence](docs/replacement-readiness.md).
