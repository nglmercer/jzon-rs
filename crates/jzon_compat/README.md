# jzon-rs-compat

Strict compatibility facade for **serde_json 1.0.151**. Every public operation, type, module and macro is an upstream re-export. Serialization/deserialization callbacks run once; errors and reader/writer streaming behavior come directly from upstream. This package retains serde_json and does not claim native acceleration.

```toml
serde_json = { package = "jzon-rs-compat", version = "0.4" }
```

The local implementation is unreleased. Dependency renaming preserves upstream Value/Number/Error interoperability; the isolated fixtures test this alongside a downstream crate using real serde_json. This package is not a same-name crates.io patch.

`std` is the default. For no_std with an allocator, disable defaults and enable `alloc`. `raw_value`, `preserve_order`, `arbitrary_precision`, `float_roundtrip` and `unbounded_depth` forward to upstream. Feature unification is safe because all APIs remain upstream. `unbounded_depth` exposes explicit opt-out methods and leaves default recursion limits intact.

Legacy `simd`, `simd-intrinsics`, `unstable`, `stats`, `fast-float` and `zmij-float-ser` flags have no effect in this facade. For native scanning/serialization use jzon-rs-serde and assess its separate contract. `jzon::compat` also delegates but belongs to the std-only core crate.

Reader APIs require EOF to conclude a document; use upstream StreamDeserializer with suitable framing for multiple documents. Writer APIs stream and can leave partial output on failure. Unsized Serialize arguments follow upstream bounds.

See [readiness](../../docs/replacement-readiness.md), [benchmarks](../../BENCHMARKS.md), and [upstream documentation](https://docs.rs/serde_json/1.0.151/serde_json/). MIT attribution and the repository LICENSE are retained. Publication ownership has not been assumed.

## Required safety override for the planned 0.4.0 distribution

The selected serde_json source is the audited local 1.0.151 patch in
`vendor/serde_json`, not unmodified registry 1.0.151. At the consumer workspace
root, use `[patch.crates-io] serde_json = { path = "/path/to/audited/serde_json" }`.
This unifies upstream type identity and protects public constructors/streams.
Cargo strips dependency paths from packages: registry-only release readiness is
blocked until a verified safe upstream release or a migrated maintained fork
exists. See replacement-readiness for exact source hashes and package checks.
