# Native optimization implementation and evidence

The six proposed optimizations are implemented and validated for the tested std-enabled native configuration. This follow-up changes Mode A's shared string emitter and Mode B's native implementation. Mode C continues to delegate upstream once; it receives no independent native acceleration claim. The broader deployment and replacement gates in [replacement readiness](replacement-readiness.md) remain open.

## Source identity

Baseline: `94727f756af4db62c6ce5892ea4b650d098622be` (implementation `82c496e`, followed by report annotation). Optimization implementation commit: **`716b782fb68d97a7d8b505f2e2e246a3f2b411c1`**, on `repair/replacement-readiness`. Measurements and validation were collected before that commit; their recorded HEAD/dirty flags are historical evidence. This commit annotation is a documentation-only follow-up and does not change validated production source. [Provenance](evidence/optimizations/provenance.json) records both production-source fingerprints. Per-invocation metadata also fingerprints all Rust source, including benchmark/tests, and records the dirty worktree honestly. The isolated baseline has only a depth measurement harness and a focused failing i128 test added; its production source is unchanged. Both checkouts use the same root Cargo.lock.

Host: AMD BC-250, Linux x86_64, Rust/Cargo 1.97.1, system allocator, empty RUSTFLAGS. Benchmarks use `float_roundtrip`, reference **serde_json 1.0.151**, and each implementation's own emitted-byte denominator. CPU, compiler, lockfile and dataset hashes are in every measurement's metadata. Baseline failing i128 serialization exited 101 with `i128 is not supported`; [reproduction log](evidence/optimizations/numeric128-baseline.log) is preserved. Native i128/u128 endpoint serialization is now implemented through the fallible sinks and tested, including partial writer errors.

## Implementation status

`VERIFIED` means the specified change passed finite checks below, not universal Serde equivalence or a speed guarantee on every workload.

| Optimization | Status | Implementation and executed evidence |
|---|---|---|
| 1. String emission and short dispatch | VERIFIED | `ser.rs` reserves before the opening quote, uses scalar scanning below one block, shares a fallible escaping implementation with Mode A, and batches validated short keys with quotes/colon into a bounded stack buffer. All bytes are checked, including Unicode/control/quote/backslash names. Threshold/output comparisons and byte-budget writer regressions pass; component and Twitter/Canada benchmarks executed twice. |
| 2. Dedicated native structs | VERIFIED | `FieldsSerializer` removes map-key dispatch and enum-wrapper state from ordinary fields; a separate struct-variant wrapper closes exactly once. Small dispatch helpers inline across crate boundaries. Derived/custom/escaped-name structs, enum representations, raw/number protocols and benchmark correctness preflights pass. |
| 3. Configurable reusable serializer | VERIFIED | `Serializer::new`, `with_capacity`, `from_vec`, `serialize`, `clear`, `buffer`, `capacity` and `into_inner`; direct `to_bytes_in` remains available. Append/reset, partial error/unwind, capacity retention and unsized standalone consumer checks pass. Warmed serialization allocates zero times in the instrumented fixture; genuine reuse has its own benchmark. |
| 4. Reusable transient string scratch | VERIFIED | `Scanner::read_str_with_scratch` shares decoding with the owned core path. Higher-ranked transient lifetimes prevent references escaping; plain strings still call `visit_borrowed_str`, escaped strings still call `visit_str`. Oversized scratch is released above 64 KiB on return/error/unwind. Differential callbacks, invalid strings, oversized/panic recovery, Miri and instrumented scaling pass. Fresh escaped String parsing can be slower; measured tradeoffs appear below. |
| 5. Fallible native streaming writer | VERIFIED | Sealed `SerializeSink` statically specializes Vec and `WriterSink<W>`, with shared escaping/numeric code. Native `to_writer` writes directly and propagates failures before later user callbacks. No whole-output intermediary. `to_writer_buffered` explicitly preserves the prior contract. Short/interrupted writes, every tested byte budget, partial output, user failures/panics, 128-bit numbers and private-token protocols pass. |
| 6. Container/depth bookkeeping | VERIFIED | Private `with_depth` guard exclusively borrows the whole deserializer and restores a local budget infallibly on success/error/unwind, avoiding Arc/atomic cost for native typed parsing. Public Scanner guards retain independent safe ownership. Default/explicit-disabled boundaries, ignored/raw oracle policies and panic recovery pass, including bounded scalar Miri. Component depths 1/16/64/127 and public-guard overhead measured. Empty native containers allocate zero times. |

No unsafe code was introduced. Safe bounded stack slicing, checked UTF-8 and existing syntax validation remain in use. Public Scanner owner-drop safety is retained. Ignored/raw skipping deliberately keeps the pinned reference's iterative lexical depth behavior, distinct from typed nesting.

## Executed validation

All **33** commands in the five machine-readable suites exited **0**, on the final production source. Command strings, status and full logs are retained under `evidence/optimizations/validation-*`:

- [Core: 15 commands](evidence/optimizations/validation-core-results.json): formatting, workspace/doctests, warnings-as-errors Clippy, documentation, four feature combinations, both wrappers, isolated consumers including negative compile probes, genuine alloc-only target, wrapper no-default checks, and benchmark preflight.
- [Safety: 2 commands](evidence/optimizations/validation-safety-results.json): `cargo +nightly miri test -p jzon-rs --test depth_safety --test readiness --test optimizations` in the bounded scalar configuration; separately nightly workspace portable SIMD tests. Miri exited 0 after approximately 162 seconds.
- [MSRV: 4 commands](evidence/optimizations/validation-msrv-results.json): standalone facade/native/alloc/derive consumers checked with Rust **1.71.0** and locked dependencies.
- [Release: 2 commands](evidence/optimizations/validation-release-results.json): `cargo audit --json`, `cargo package --workspace --allow-dirty`; no publishing.
- [Fuzz: 10 commands](evidence/optimizations/validation-fuzz-results.json): native, ignored skip, raw capture, roundtrip and SIMD targets, both default and roundtrip policy, each bounded to 5 seconds / 4096 bytes / 512 MiB. Roundtrip fuzz now compares direct streaming and reusable output too. These are smoke campaigns, not long campaigns.

`tests/optimizations.rs` adds nine default tests, ten with raw_value, and is included in CI's targeted Miri command. Existing conformance, readiness, depth and scalar/SIMD differential coverage remain active. Standalone native consumer probes compile the new reusable APIs and named writer serializer type without needing an oracle dependency.

Additional allocation and benchmark commands are recorded separately rather than included in the 33 count. Final benchmark campaigns run sequentially after build/test/fuzz/Miri/package work finished. Initial candidate and inlining-stage measurements are preserved with `intermediate-` and `inlined-` prefixes; they are **not** current results and are never substituted as historical bests. The allocation fixture's single-sample times are diagnostics, not Criterion speed evidence.

**NOT RUN:** perf/valgrind call-graph profiling (not installed); Windows/macOS/aarch64 runtime jobs (no local environments and no push); long fuzz campaigns; every CPU's intrinsic dispatch path. Component profiling and compiled-symbol inspection were performed. Native no_std and streaming reader parsing remain missing capabilities. Native public errors remain different from upstream. None of these is repaired merely by documenting it.

## Allocation tradeoffs

[Current instrumented allocation data](evidence/optimizations/allocation.csv), compared to the committed repaired baseline `evidence/allocation.csv`:

| Short escaped strings | Before allocations | After allocations | Before allocated bytes | After allocated bytes | After retained String capacities |
|---:|---:|---:|---:|---:|---:|
| 100 | 202 | 102 | 4,796 | 3,188 | 100 |
| 1,000 | 2,002 | 1,002 | 41,600 | 25,592 | 1,000 |
| 10,000 | 20,002 | 10,002 | 563,240 | 403,232 | 10,000 |

At 10,000 items this is approximately **50% fewer allocations** and **28.4% fewer allocated bytes**. Destination-owned Strings still require their own allocation; transient decoding now reuses scratch. Both retained capacity and allocation totals scale approximately linearly. Mode A still retains roughly 16 bytes per tiny escaped String. Scalar/borrowed parsing, warmed transient scratch, warmed reusable serialization, direct streaming to an allocation-free sink and empty native containers each have zero allocations in the fixture.

Long individual late/dense/Unicode escaped strings can still require both scratch and destination storage, so they may allocate more total bytes than Mode A. Scratch above 64 KiB is freed after each operation to avoid permanently retaining a document-sized buffer; that memory bound deliberately limits long-string reuse. No new peak-RSS measurement was collected for this follow-up, and allocated bytes are not presented as peak resident memory.

## Current PC measurements

All eight final benchmark commands exited **0**. [Commands](evidence/optimizations/verified-commands.json), raw estimates including samples, metadata and full logs are retained with the `verified-` prefix. Reproduce with:

```sh
python3 tools/benchmark-optimizations.py --baseline /path/to/isolated-94727f7 --out /path/to/new-results --repeats 2
cargo run --release --manifest-path fixtures/allocation/Cargo.toml --locked
```

Component timings use 30 samples / 0.2 s warmup / 0.5 s measurement; larger serialization cases use 200 samples / 1 s warmup / 4 s measurement. These shared-PC measurements are exploratory. The review threshold is repeatable >=5% improvement with non-overlapping 95% intervals for equivalent contracts. The ranges below span the two actual invocation means, not confidence intervals; confidence intervals are preserved in each raw report. Speed = paired **before time / after time**; below 1x is slower. Neither best-of-history selection nor mixing Mode A/B/C is used.

| Workload | Before time/op (two-run range) | After time/op (two-run range) | Paired speed ratio |
|---|---:|---:|---:|
| B wide struct, fresh serialization (104 bytes) | 282.93–285.65 ns | 164.94–181.45 ns | **1.57–1.72x** |
| B wide struct, direct buffer reuse | 274.61–277.67 ns | 152.83–185.01 ns | **1.48–1.82x** |
| B wide struct, explicit buffered writer | 285.69–293.05 ns | 171.67–184.07 ns | **1.59–1.66x** |
| B wide struct, parsing/field dispatch | 890.95–893.46 ns | 716.93–867.64 ns | 1.03–1.25x |
| B plain owned String parsing | 45.25–45.37 ns | 40.39–42.27 ns | **1.07–1.12x** |
| B fresh dense escaped String parsing | 74.54–75.53 ns | 84.12–85.35 ns | **0.89x** |
| B fresh late-escape String parsing | 57.21–58.03 ns | 62.83–64.60 ns | **0.89–0.92x** |
| B depth 1, typed Value parsing | 85.76–90.93 ns | 73.05–79.10 ns | **1.08–1.24x** |
| B depth 127, typed Value parsing | 9.018–9.162 us | 8.430–8.736 us | 1.03–1.09x |
| A projected Twitter serialization (51,013 bytes) | 50.18–51.38 us | 42.13–42.21 us | **1.19–1.22x** |
| B projected Twitter serialization (51,013 bytes) | 90.50–93.82 us | 82.36–83.00 us | **1.09–1.14x** |
| A Canada serialization (2,090,326 bytes) | 4.363–4.433 ms | 4.394–4.395 ms | 0.99–1.01x |
| B Canada serialization (2,090,326 bytes) | 3.768–3.863 ms | 3.545–3.554 ms | **1.06–1.09x** |

Wide serialization, plain owned parsing, shallow depth and the stated A/B dataset gains meet the exploratory repeat threshold. Wide parsing and deeper nesting do not show a repeatable >=5% gain; public guard cost is essentially unchanged. Dense/late escaped fresh Strings regress despite fewer allocations in arrays. Integer/float and escaped emission component results vary across repeats, so no acceleration claim is made there.

The new owned reusable wide serializer measures 155.13 / 171.44 ns; streaming into a reused Vec measures 229.90 / 236.35 ns. Those APIs have distinct contracts from full buffered writing, so they are not relabelled as the same case or assigned a misleading baseline ratio. Streaming removes the intermediary allocation and changes I/O error/callback timing; it does not guarantee the fastest Vec emission path.

For reference comparisons, both final invocations measured these same output bytes:

| Serialization workload | Mode A time | Mode B time | Reference time | A speed vs reference | B speed vs reference |
|---|---:|---:|---:|---:|---:|
| Projected Twitter | 42.13–42.21 us | 82.36–83.00 us | 48.39–48.73 us | 1.15x | **0.58–0.59x** |
| Canada | 4.394–4.395 ms | 3.545–3.554 ms | 3.424–3.750 ms | 0.78–0.85x | **0.97–1.06x** |

The Canada reference varied enough to reverse which engine was faster in the second invocation. That is no robust native advantage over upstream. Mode B Twitter remains substantially slower than reference even after improvement over its own baseline. Mode C was unchanged and was not rerun in this optimization campaign; its prior delegated measurements remain explicitly historical. No global 2x claim or independent facade speed claim is supported.

Full final reports: [components before 1](evidence/optimizations/verified-components-before-1.md), [after 1](evidence/optimizations/verified-components-after-1.md), [before 2](evidence/optimizations/verified-components-before-2.md), [after 2](evidence/optimizations/verified-components-after-2.md); [datasets before 1](evidence/optimizations/verified-serialize-before-1.md), [after 1](evidence/optimizations/verified-serialize-after-1.md), [before 2](evidence/optimizations/verified-serialize-before-2.md), [after 2](evidence/optimizations/verified-serialize-after-2.md).

## API and migration effects

Native `to_writer` now streams: I/O failure may stop callbacks earlier, and partial output remains visible. Call `to_writer_buffered` when the previous buffer-before-I/O behavior is required. Neither operation flushes the supplied writer. Use a caller-owned BufWriter when system-call batching is appropriate; no identical-write-boundary promise is made. Native `from_reader` still buffers all input and has its documented EOF/error limitations.

`Serializer` and native container types now accept a defaulted sink parameter; usual `Serializer` names remain Vec-backed. `StructSerializer::Map` contains `FieldsSerializer`, and the associated struct-variant serializer type changed. Code explicitly naming these container internals may need adjustment. Recommend the existing planned **0.4.0** release for this combined 0.x API work; no version bump/publication occurred.

For reuse, `serialize` appends, `clear` retains capacity, and errors/panics retain bytes already emitted. Parser input position is retained after errors/panics rather than transactionally rolled back. Escaped transient borrows cannot survive the visitor call; plain input borrowing remains tied to the input lifetime.

This completes the six scoped optimization implementations for tested std native APIs. It does not certify unrestricted replacement, remove upstream from the facade, implement native no_std/streaming readers, or close remote platform gates.
