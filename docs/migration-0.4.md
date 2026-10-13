# Migration to planned 0.4.0

All four packages use 0.4.0. This is a local breaking-change preparation, not a
published registry release.

Native Serde errors remain `jzon::serde_impl::Error`, distinct from upstream.
Parser errors now wrap their cause with byte-based `line()` / `column()` and
`classify()` (`Io`, `Syntax`, `Data`, `Eof`). Use `cause()` when matching an old
underlying variant. Serializer errors have no input position (0, 0). Mode A's
`jzon::Error` is unchanged. Messages and every error position are not certified
identical to upstream.

Native `from_reader` now parses incrementally, invokes callbacks once, retries
Interrupted reads, and reports I/O failures as encountered. Use
`from_reader_buffered` / `from_reader_buffered_with_stats` for the former
read-to-end contract. A reader cannot lend borrowed input to a visitor; use
`DeserializeOwned`. `ReaderDeserializer::new(reader).into_iter::<T>()` parses
owned values; `Deserializer::from_slice(bytes).into_iter::<T>()` supports borrowed
values. Native streams stop after an error. Their offsets describe actual
consumption, including failed parsing, rather than upstream's recovery offset
contract. To recover an underlying reader, use `into_parts()` and retain its
optional already-read lookahead byte. Parser storage follows scalar-token length
and nesting, not total document length. A requested RawValue needs its own raw
capture; scalar strings are not subject to a fixed input-size cap.

Core and native wrapper default to `std`. Disable defaults and enable `alloc`
(and `serde` / `derive` on the core as needed) for no_std. Generated code uses
core/alloc paths. I/O APIs, HashMap support and runtime CPU detection require std;
public guard state requires pointer-width atomics. Native-only mirrors forward
weakly and do not introduce an upstream JSON dependency. `preserve_order` enables
std; the native engine streams user map order and does not define its own Value.

Both strict compatibility facades still re-export the same upstream types.
The reviewed registry serde_json 1.0.151 has a confirmed non-ASCII boolean-key
safety defect. The audited source in `vendor/serde_json` retains package identity
and fixes initial-byte handling; it is **not** a new upstream fixed release.
Source consumers must select that source, and independent consumers using
serde_json themselves need the same workspace-root override to unify types:

```toml
[patch.crates-io]
serde_json = { path = "/absolute/path/to/jzon-rs/vendor/serde_json" }
```

Cargo strips facade path dependencies when packaging. Extracted packaged artifacts
were checked with this override and reject affected cases under Miri; without it,
metadata resolves the unsafe registry package. Transitive workspace patches do not
protect a downstream root. Do not distribute a registry-only release until a
verified safe release or a deliberately migrated maintained fork resolves this
blocker. Preserve the exact pin and review any future source update under Miri,
consumer interoperability and package checks.

See [replacement readiness](replacement-readiness.md) for executed validation,
measured reader regressions, source hashes and separate native/facade conclusions.
