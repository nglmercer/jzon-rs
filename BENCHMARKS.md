# Benchmark methodology and current results

Mode A custom derives, Mode B native Serde and Mode C delegated facade are separate products. `bench_cmp` measures A/B; `bench_compat` measures the actual facade package. Native Twitter/CITM schemas project selected fields, whereas the facade benchmark materializes full Value trees. These workloads cannot be compared as interchangeable full-document parsing.

Every serialization case computes its own emitted bytes outside timing. Preflight validation compares serialized JSON values against the reference for the same typed value; Mode A's documented float zero token `0` is accepted as numerically equivalent to `0.0` only within the exactly representable integer range. Output lengths are logged. Mode A's mixed typed schema and full Value benchmarks differ in output size and must remain separately identified. Immutable simd-json tests are labelled copy_then_parse because input copying is timed. Native `to_writer` streams directly; `to_writer_buffered` is explicitly buffered; `to_bytes_in` is actual buffer reuse. Sonic `to_string` is labelled fresh allocation.

The workflow records compiler, commit, worktree patch, lockfile, dataset hashes, features, OS/CPU, allocator, sample settings, estimates and confidence intervals. Each invocation uses fresh results; historical high-water data is archived in [benchmark-history](docs/benchmark-history.md) and is never merged. CI uploads artifacts without editing READMEs or generating maximum speedup headlines.

Review thresholds declared for this repair: require repeatable >=5% time/op improvement with non-overlapping 95% intervals in equivalent contracts before describing a speed improvement. Short/shared-host runs are exploratory, not deployment guarantees. Allocation improvements use workload scaling and retained capacity rather than an exact allocator-specific capacity threshold. Do not infer platform results from this x86_64 host.

Commands:

```sh
CRITERION_HOME=bench_results/native cargo bench -p jzon-rs --bench bench_cmp --features float_roundtrip -- '^serialize/(twitter|canada)'
CRITERION_HOME=bench_results/compat cargo bench -p jzon-rs-compat --bench bench_compat
cargo run --release --manifest-path fixtures/allocation/Cargo.toml
```

Current executed results and their limitations are in [replacement-readiness](docs/replacement-readiness.md). No blanket acceleration claim is supported.

Current optimization measurements are tracked separately in [optimization progress](docs/optimization-progress.md); earlier readiness figures describe their recorded source fingerprints.
