# jzon-rs-compat

[![crates.io](https://img.shields.io/crates/v/jzon-rs-compat.svg)](https://crates.io/crates/jzon-rs-compat)
[![docs.rs](https://docs.rs/jzon-rs-compat/badge.svg)](https://docs.rs/jzon-rs-compat)
[![MSRV](https://img.shields.io/badge/rustc-1.71%2B-blue.svg)](https://blog.rust-lang.org/2022/11/03/Rust-1.71.0.html)

Drop-in replacement for `serde_json` via a dependency rename.

## Setup

One line per crate in `Cargo.toml` — no code changes required:

```toml
[dependencies]
serde_json = { package = "jzon-rs-compat", version = "0.3" }
```

For a workspace, apply the rename in every member that uses `serde_json`:

```sh
for f in crates/*/Cargo.toml; do
  sed -i 's/^serde_json = /serde_json = { package = "jzon-rs-compat", version = "0.3" } # /' "$f"
done
```

(Adjust to taste; the point is each `serde_json` dependency line becomes a
rename. `cargo build` will tell you about any line the script mangled.)

## Feature mapping

`serde_json` features map 1:1 to same-named flags here — move them onto the
renamed line instead of enabling `serde_json`'s directly, so the engine and
the fallback stay in agreement:

```toml
serde_json = { package = "jzon-rs-compat", version = "0.3", features = ["arbitrary_precision"] }
```

| Flag | Default | Description |
|------|---------|-------------|
| `fast-float` | on | `ryu` float serialization, `fast_float2` parsing |
| `simd` | off | u128 SWAR scanning (16 bytes/iter) |
| `unstable` | off | `std::simd` portable SIMD 32–64 bytes/iter (nightly only) |
| `stats` | off | Allocation counters on the underlying Scanner |
| `arbitrary_precision` | off | Exact big numbers (engine + fallback agree) |
| `preserve_order` | off | Insertion-ordered `Map` (needs a toolchain that builds current `indexmap`; the rest of the crate is MSRV 1.71) |
| `raw_value` | off | `value::RawValue` / `to_raw_value` |
| `float_roundtrip` | off | Exact float parsing in the fallback |
| `unbounded_depth` | off | No 128-level recursion limit in the engine |
| `std` / `alloc` | on/n/a | No-ops: this crate is std-only, so renames stay total |

To disable the fast-float default: `serde_json = { package = "jzon-rs-compat", version = "0.3", default-features = false }`.

## What it does

- Routes `from_str` / `from_slice` / `from_reader` / `to_string` / `to_vec` /
  `to_writer` through jzon's SIMD engine.
- On any engine error, retries with real `serde_json`, so values, error
  messages, line/column positions, and error categories are authoritative.
- Re-exports all `serde_json` public types (`Value`, `Map`, `Number`,
  `Error`, …) and modules (`de`, `ser`, `error`, `map`, `value`) unchanged,
  so renamed crates interoperate with third-party crates still on real
  `serde_json`: the types are literally identical, values cross with zero
  conversion.

## Known differences

- **Float parsing is always correctly rounded** (equivalent to `serde_json`
  with `float_roundtrip`). Default `serde_json` uses a faster approximate
  algorithm that differs by 1ulp on rare inputs (e.g. `5e-77`). The engine
  deliberately does not replicate the approximation; enable
  `float_roundtrip` on both sides for bit-identical parses.
- **Whole-tree replacement is not supported** (see below); rename per crate.

## Why not `[patch]`?

Two cargo mechanics, both verified against cargo 1.97 with fixture crates,
rule out a one-line whole-tree patch:

1. **Renamed patches are silently ignored.** `[patch.crates-io] serde_json =
   { package = "jzon-rs-compat", … }` resolves the patch by its package name
   (`jzon-rs-compat`), finds no such dependency in the graph, and warns
   `patch … was not used in the crate graph` while continuing to build real
   `serde_json`.
2. **A same-name vendored patch is ignored too.** Copying this crate to
   `vendor/serde_json` with `name = "serde_json"` still warns "not used":
   the crate itself depends on real `serde_json` (fallback + type
   re-exports), so it can never replace its own dependency.

Whole-tree replacement is therefore not supported; rename per crate. The
literal type identity above is what makes the mixed tree seamless.

## Part of the jzon family

| Crate | Purpose |
|-------|---------|
| [jzon-rs](https://crates.io/crates/jzon-rs) | Core zero-copy JSON with `#[derive(ToJson, FromJson)]` |
| [jzon-rs-serde](https://crates.io/crates/jzon-rs-serde) | SIMD-backed serde `Serializer`/`Deserializer` |

## License

MIT
