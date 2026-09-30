# Replacement-readiness assessment

## PR #1 remaining fixes — 2026-09-30

This section supersedes the current conclusions in the historical reports below.
The checkout started clean at **1ef7809435e1ce57bc3198ef560e7774f74307c7**, matching
the reviewed baseline. No repository or parent AGENTS.md was found. The named
`JZON_PR1_REMAINING_FIXES.md` was absent; the user's subsequently supplied full
specification governed this implementation. Existing native map-key, guard,
container, allocation and callback repairs were preserved. Validation uses the
local modified source, bound by the resulting local commit and
[source fingerprints](evidence/pr1/source-identity.json). No reset, push, merge,
publication, billing or security-setting change occurred.

Remote CI: **EXCLUDED_BY_USER — billing limitation**. There is no remote approval,
dispatch or final-head CI gate in this local task. Earlier remote-status statements
below are historical evidence only.

### Implementation status and separate conclusions

| Work item | Implementation and locally observed status | Concrete limits / blockers |
|---|---|---|
| Upstream safety | Audited local serde_json **1.0.151** bool-key patch, shared by both facades; affected constructors and streams checked under Miri. Direct string fuzz comparisons restored after validating the patched reference. | No verified fixed upstream registry release was selected. This is a narrow audited patch, not a safety proof for all upstream code. |
| Shipped dependency | Active manifests, independent fixtures, fuzz and resolved lockfiles select the safe source. Independent source and extracted-package consumers verify its exact `de.rs` hash and a single shared upstream type identity under Miri. | Cargo strips path dependencies in packaged manifests. **Consumers must override serde_json at their workspace root.** Without that override, package metadata demonstrably resolves the unsafe registry source. Registry-only release is blocked. |
| Native errors / dispatch | Native Serde has category, byte-based line/column, wrapped cause and public-constructor positions. Valid wrong types are Data errors; malformed syntax and EOF remain distinct. Visitor errors retain precedence. Byte visitors preserve native byte/surrogate rules. | Native messages/types remain distinct. Exhaustive upstream error-message/position equivalence is not certified; Mode A keeps its existing error type. |
| Native readers / streams | Incremental native parser, fragmented reads, Interrupted retry, EOF/I/O handling, explicit container completion, primitive boundaries, owned reader streams and borrowed slice streams. No whole-document reader buffer or callback replay. | Scalar tokens require token-sized storage; RawValue explicitly captures its requested raw value. Streams fuse on errors and report actual parser consumption, not upstream's recovery-offset contract. |
| Native no_std + alloc / features | Core and native wrapper gate std/I/O/HashMap paths; generated code uses core/alloc. Native Serde and Mode A derive compile on **thumbv7em-none-eabi**, including Rust **1.71.0**. Native mirror features do not activate serde_json. | Public guard storage requires pointer-width atomics; targets without them are outside this demonstrated support. Runtime SIMD/reader APIs require std. Other architectures and platforms are not locally executed. |
| Validation / performance | Formatting, Clippy, complete local suites, consumers, MSRV, targeted Miri, typed and Value reader fuzzing, package checks and repeated equivalent reader benchmarks recorded below. | Finite local campaigns are not deployment certification. CPU-callgraph profiling tools are unavailable; no speculative timing optimization or gain is claimed. The incremental reader has a measured timing regression. |
| Migration / release preparation | All internal packages and version requirements aligned to planned **0.4.0**; [migration notes](migration-0.4.md), changelog and [PR description](pr1-description.md) prepared. | Registry-only distribution remains blocked by safe-source resolution. No release was published. |

**Native completeness:** the requested incremental-reader, structured-error and
alloc implementations now exist and have finite local coverage; unrestricted
replacement remains uncertified, especially exact error/recovery contracts and
unexecuted platform/deployment workloads. **Facade compatibility:** both strict
surfaces remain direct upstream re-exports, with shared type interoperability
when consumers select the audited source. **Measured acceleration:** this reader
workload regresses; historical serialization comparisons retain their own scope.
**Dependency removal:** native-only consumers contain no upstream JSON dependency;
facades still require upstream JSON and are not acceleration or dependency removal.
**Registry-release readiness:** blocked without a verified safe registry source or
an explicitly migrated maintained fork. A transitive workspace patch cannot fix
that distribution requirement.

### Audited reference and packaging evidence

The original registry string-key reproduction was run **only under Miri** and
failed with Undefined Behavior, exit 1. Its
[probe](evidence/pr1/original-oracle-probe.rs),
[manifest](evidence/pr1/original-oracle-Cargo.toml),
[lockfile](evidence/pr1/original-oracle-Cargo.lock) and
[observed failure](evidence/pr1/upstream-baseline-miri.log) are retained.
`vendor/serde_json` preserves upstream package name/version and MIT/Apache
licenses. The [patch](evidence/pr1/upstream-safety.patch) peeks at the first
boolean-key content byte and consumes it only on literal `t` / `f` paths. The
invalid-key branch parses the complete string, preserving StrRead's UTF-8
precondition. It does not decode escapes into accepted boolean spellings or
replay callbacks. The [source record](evidence/pr1/dependency-source.json) gives
original and patched hashes and the update policy. No newer release is assumed
safe. The inspected upstream source was not a verified fixed release.

The vendor manifest also pins optional **indexmap =2.2.3**, a real release with
Rust 1.63 metadata. The initial mirror MSRV attempt resolved 2.14.2 and failed
because Cargo 1.71 cannot read its edition-2024 manifest. The corrected mirror
consumer passed on Rust 1.71; both failure and correction logs remain recorded.
This manifest constraint is explicit additional patch scope, separate from the
three-line boolean-parser repair.

`tools/check-safe-dependency.py` constructs an independent consumer with both
facades, upstream Value interoperability, public string/slice/reader constructors
and streams. Its packaged mode extracts all four `.crate` artifacts plus the
safe serde_json artifact. It first checks **metadata only** without a serde_json
override, proving unsafe registry resolution without executing that source; it
then installs the required root override, proves a single safe source and hash,
and executes the affected consumer under Miri. The selected source survives
packaging **with this explicit override**. Packaging alone does not embed it.
See [package command results](evidence/pr1/package-results.json) and
[packaged consumer log](evidence/pr1/packaged-consumer-final.log).

### Native implementation and storage contract

`ReaderDeserializer` drives containers incrementally with a single lookahead
byte, RAII depth restoration and a reusable scalar token. Its lifetime bridge
converts borrowed scalar callbacks to transient string/byte callbacks; it invokes
user seeds and visitors once and contains no unsafe lifetime extension. Scalar
parsing delegates to the **native slice engine**, never upstream. Ignored/raw
values use iterative lexical grammar with validated keys, escapes and controls.
Ignored-token errors retain their local positions; large lexical token storage is
released after completion. Requested raw captures are output requirements, not a
hidden general document buffer. Tests parse a 100,000-element array with a maximum
three-byte scalar token, exercise deep ignored structures and retain byte/surrogate
behavior separately from String behavior.

`from_reader_buffered` and its stats variant retain the explicitly named former
contract. `ReaderDeserializer::into_parts()` returns a prefetched byte together
with the source so partial consumption is not silently lost. Reader stats count
native scalar work actually performed; they do not claim borrowed output or
complete scanner-event accounting for reader structural/ignored traversal.
Slice and reader streams validate primitive boundaries (for example `truefalse`
is rejected) and stop after errors. Early visitor completion cannot hide malformed
container tails. Reader depth restoration is tested across visitor panic unwinding.

### Exact local commands and results

[Validation results](evidence/pr1/validation-results.json),
[additional results](evidence/pr1/extra-results.json),
[Miri results](evidence/pr1/miri-results.json),
[package results](evidence/pr1/package-results.json),
[fuzz results](evidence/pr1/fuzz-results.json) and
[measurement results](evidence/pr1/measurement-results.json) contain exact argument
vectors, exit codes and elapsed times; adjacent logs retain output messages.
Captured logs normalize trailing whitespace and redundant final blank lines for
repository whitespace checks; result content is unchanged.
Historical/intermediate failures retain their original labels. The generated root
lockfile is normally ignored by this library repository; its tested resolved
snapshot is preserved as [workspace lock](evidence/pr1/workspace-Cargo.lock).
Tracked independent fixture and fuzz lockfiles were updated; fixture lockfiles
remain Cargo-format 3 for the supported MSRV.

| Command / configuration | Observed result |
|---|---|
| `cargo fmt --all -- --check`; native alloc fixture formatting | Passed. |
| `cargo clippy --workspace --all-targets --features 'compat,raw_value,preserve_order,arbitrary_precision,float_roundtrip,unbounded_depth,simd,simd-intrinsics,stats,zmij-float-ser' -- -D warnings` | Passed; checks retained. Initial feature-combination lints were fixed in test data/imports/assertions. |
| `cargo test --workspace`; complete mirror, SIMD and strict configurations | Passed, including direct native regression tests and doctests. |
| `cargo +nightly test --workspace --features serde,simd,unstable` | Passed on local x86_64. |
| `cargo test -p jzon-rs --features serde,raw_value --test native_reader --test native_errors` | **13 reader + 2 error tests passed**. Existing eight PR21 key groups pass in complete configurations. |
| `python3 tools/check-consumers.py` | Passed all independent source consumers, real native no_std target and both expected compile rejections. |
| `cargo +1.71.0` facade/mirror, native, derive, alloc and native-alloc target fixture checks | Passed with locked dependency resolution; exact commands in results. |
| Native wrapper alloc/mirror and strict-facade alloc target checks on `thumbv7em-none-eabi` | Passed. Normal native dependency tree contains **no serde_json**. |
| `cargo doc --workspace --no-deps --features compat,serde` | Passed. |
| Targeted Miri, strict provenance | **36 final-source guard/error/key/readiness/facade tests + 12 reader tests passed**, exit 0; one large reader storage test excluded from Miri and passed normally. |
| `cargo package --manifest-path vendor/serde_json/Cargo.toml --allow-dirty`; `cargo package --workspace --allow-dirty` | Passed, including Cargo package build verification. |
| `python3 tools/check-safe-dependency.py`; same with `--packaged` | Passed affected probes under Miri and verified safe-source hash/type identity. Packaged negative metadata probe confirms the release blocker. |
| Typed `map_keys` fuzz, default and roundtrip policy; `native` Value reader fuzz | **50,541 typed default / 48,648 typed roundtrip / 210,190 Value reader executions**, all exit 0; direct patched string comparisons. |

The earlier full Miri run passed **46 tests**, including all **10 optimization**
tests, before the final stream-boundary and ignored-reader position changes. Its
[result record](evidence/pr1/miri-before-final-source-results.json) and adjacent
logs retain that snapshot scope. The final-source run repeats the relevant guard,
error, key, readiness, facade and reader tests; it does not needlessly repeat the
unchanged large optimization test.

Fuzz campaigns are bounded smoke tests (4096 bytes, 512 MiB, 30 seconds), not long
coverage or a correctness proof. Affected upstream string probes in the dedicated
test module are `cfg(miri)`; direct oracle fuzzing is enabled only after the
patched source's Miri verification. The large document-storage reader test runs
normally but is excluded from targeted Miri to keep the interpreter campaign
bounded; this limitation is explicit, not a fabricated Miri result.

### Measured reader performance and remaining validation limits

The two final measured means (full Vec<u32>, 0..10,000) are:

| Engine | Run 3 | Run 4 |
|---|---:|---:|
| Native incremental | 1,401.2 us | 1,133.8 us |
| Native explicitly buffered | 521.49 us | 523.76 us |
| Audited upstream reader | 440.05 us | 317.92 us |

The incremental reader takes approximately **3.2–3.6 times the reference time**
on this workload. Both measured configurations are slower than the reference;
variation across runs prevents claiming a stable exact ratio. Criterion's
historical change percentages compare earlier exploratory runs and are not
attributed to an optimization in this patch. The implementation meets the
incremental storage contract, not a universal speed claim.

Allocation counts measure allocator calls (including reallocations) and total
requested bytes, separately from timing, input/output storage and peak live
memory. Inputs and expected output are constructed outside the measured region.
At 10,000 elements (48,891 input bytes), incremental native parsing measured
**14 allocation/reallocation calls, 131,064 requested bytes**; buffered native
**14 / 179,947**; audited upstream **13 / 131,056**. This includes owned Vec output
allocation, not input construction. Native ignored traversal measured **3 calls /
17 requested bytes** at 100, 1,000 and 10,000 elements. These are totals requested,
not peak resident memory or allocator-retained capacity.

Benchmarks use equivalent full-reader work and identical 48,891-byte input;
reader throughput uses **input bytes**, while existing serializer reports must
continue using emitted output bytes. Repeated final runs use 30 samples, one-second
warm-up and two-second measurement, pinned to CPU 10 with Miri excluded from that
CPU. Shared host load/cache/frequency effects remain possible. Early runs 1/2 are
exploratory and lack final-source identity; they do not establish an improvement.
`perf` and `valgrind` are absent, so CPU-callgraph profiling and an evidence-backed
timing optimization remain undone. Preserve this regression and profile before
changing the parser for speed. Allocation/storage measurements are not throughput
claims. No ARM runtime, Windows/macOS, non-atomic target, long fuzz campaign or
production deployment validation ran locally. These limits and the registry
safe-source blocker prevent an unrestricted replacement/release certification.

---

## PR #21 map-key follow-up — 2026-09-30

This section supersedes earlier map-key validation, facade safety scope and remote-status statements for the current follow-up; the historical reports below retain their original source scope. Starting checkout was clean at reviewed head **`dffe27f2973149dd2515033ac6477046664bdc6f`**, with base **`f96732b96408e6355153fdebca733148678d81ab`**. No repository `AGENTS.md` instructions were found. Validation ran against an **uncommitted local working tree**. The local commit containing this follow-up records that tested patch; it is not a new GitHub head. [Source identity](evidence/pr21/source-identity.json) records SHA-256 hashes of production code, regressions, fuzz harness, workflow changes and resolved lockfiles, plus actual host/compiler versions.

The accompanying regression file was not present in the checkout or supplied as an attachment. `pr21_native_map_regressions.rs` was reconstructed from the review's cases and compared with the locally resolved **serde_json 1.0.151** oracle. Before production edits, its six initial groups all failed, exit **101**: unit missing keys, object-shaped enum keys, escaped boolean keys, byte callbacks, pre-seed rejection and custom unit visitors. [Actual baseline output](evidence/pr21/baseline.log) distinguishes observed native and reference results. These are executed failures, not source-derived predictions. The valid custom unit visitor exposed an additional mismatch: upstream delivered the quoted key as a string, whereas native visited unit without consuming the string. No oracle expectation was changed to make native pass. A seventh group subsequently added explicit boolean callback timing checks.

Repairs are confined to native map access and key handlers:

- **R1:** Check the next byte for a quote at the shared map-access boundary, before state changes or user seeds, leaving the quote for existing readers. Preserve explicit trailing-comma and EOF branches. This applies to enum, custom, newtype and raw-token paths alike. Unit and unit-struct key entry points forward to string visitors like upstream.
- **R2:** Consume only literal quoted `true` or `false`, validating the closing quote before `visit_bool`. Escaped spellings remain valid JSON strings but are rejected as typed boolean keys.
- **R3:** Delegate both byte-key entry points to ordinary native byte deserialization. Tests compare borrowed/plain and transient/escaped callbacks, lone and paired surrogates, raw non-UTF8 and control bytes, malformed escapes and post-comma grammar. Blanket UTF-8 validation was not introduced.

Eight regression groups call **`jzon::serde_impl::from_str` and `from_slice` directly**, including when `compat` is enabled. Coverage includes custom unit/boolean visitors, unit-struct and newtype dispatch, both byte entry points, actual borrowed byte keys and malformed keys after commas. The new `map_keys` differential fuzz target compares acceptance, decoded map values and byte callback provenance for unit, custom unit/unit-struct, enum, bool, newtype and both byte-entry families through both native input APIs. An eighth deterministic group covers non-ASCII boolean/newtype keys using the safe pinned slice oracle. `tools/seed-fuzz.py` seeds the typed cases and conformance corpus. CI includes the regression file in Miri; Fuzz includes the typed target under both default and roundtrip policies.

**New pinned-oracle defect observed:** Initial typed fuzz campaigns under both policies exited **1** (libFuzzer target exit 77) inside the upstream `StrRead` boolean-key error path. A separate upstream-only Miri probe for `serde_json::from_str::<BTreeMap<bool, u8>>(r#"{"é":1}"#)` exited **1** with **Undefined Behavior**, independent of native code. The pinned boolean handler consumes the first content byte before calling `StrRead::parse_str` in its invalid-key branch; for a multibyte initial character this leaves a continuation byte, while `StrRead` assumes UTF-8 and uses unchecked conversion. Formatting the resulting invalid string caused the observed Miri failure. See [isolated probe](evidence/pr21/oracle-probe.rs), [manifest](evidence/pr21/oracle-probe-Cargo.toml), and [Miri log](evidence/pr21/oracle-nonascii-miri.log). Both strict facades delegate this upstream path and inherit its limitation; earlier facade suitability statements do not establish safety for this case.

The fuzz harness does **not** skip non-ASCII inputs or change their acceptance expectations. For boolean and boolean-newtype families on non-ASCII input, it compares native `from_str` against pinned `from_slice`, which safely rejects; all other string-oracle comparisons remain direct. Native `from_slice` is always compared directly too. This explicit oracle-domain adjustment avoids invoking known upstream UB. Initial failed fuzz logs remain in the evidence, separate from final campaigns. The upstream pin is unchanged. Resolving this upstream defect and renewing facade validation is separate work required before any broader facade safety certification.

Final local commands below all exited **0**, after the final regression/harness changes:

| Validation | Observed result |
|---|---|
| `cargo test -p jzon-rs --test pr21_native_map_regressions` | **8 passed**, default configuration. |
| Same native test with `compat,raw_value,preserve_order,arbitrary_precision,float_roundtrip,unbounded_depth` | **8 passed**; direct native calls, including oracle-aware non-ASCII bool rejection. |
| Same native test with `compat,simd,simd-intrinsics,stats,zmij-float-ser` | **8 passed**. |
| `cargo test --workspace`; complete `jzon-rs` feature-mirror and SIMD suites | All passed, including doctests. |
| `MIRIFLAGS=-Zmiri-strict-provenance cargo +nightly miri test -p jzon-rs --test depth_safety --test readiness --test optimizations --test pr21_native_map_regressions` | **42 passed**: 4 depth-safety, 21 readiness, 9 optimization and 8 map-key tests. The separate oracle-only UB reproducer intentionally fails and is not part of this passing native suite. |
| `python3 tools/check-consumers.py` | All isolated facade, mirror, native, alloc, interop and derive consumers passed; both expected compile-fail probes rejected correctly. |
| Formatting, workspace/all-target Clippy with `-D warnings`, workspace docs | All passed. |
| `cargo +nightly test --workspace --features simd,unstable` | Passed on the local x86_64 host. |
| `cargo +nightly fuzz run map_keys -- -max_total_time=30 -max_len=4096 -rss_limit_mb=512` | **389,997 executions**, exit 0; bounded smoke campaign with the documented oracle adjustment. |
| Same typed fuzz command with `--features roundtrip_policy` | **361,711 executions**, exit 0; same bounds and adjustment. |

[Command results and elapsed times](evidence/pr21/validation-results.json) link each result to its complete adjacent log. The baseline, initial oracle-crashing campaigns and intentional oracle-only Miri failure remain recorded as failures; they are not relabelled as successful validation. Final source hashes were checked against the tested files. No performance benchmark, remote platform job, MSRV campaign, security audit, release packaging, long fuzz campaign or final-head GitHub check was rerun for this follow-up; earlier evidence for those commands retains its historical scope.

The actual GitHub read on this date confirms the open, non-draft PR still has the reviewed head, seven commits and 663 changed files, with an empty description. [CI status](evidence/pr21/ci-status.json) and [Fuzz status](evidence/pr21/fuzz-status.json) both report **`action_required`**, not success, for that head. Both workflow files were read in full. This follow-up only extends the Miri command and Fuzz target matrix; existing `pull_request` triggers, read-only `contents` permissions, runner matrices and campaign bounds are retained. An authorized maintainer must approve the fork PR workflow runs and obtain passing CI/Fuzz on the final pushed head. Local runs do not satisfy that gate. The user subsequently authorized a local commit. No workflow approval, alternate-event dispatch, push, merge or publication was performed. A [PR description draft](pr21-description.md) is prepared locally for the updated head.

**Separate verdicts:** R1–R3 are repaired in the local source, subject to the recorded finite checks. Merge approval remains pending final-head CI/Fuzz. Native unrestricted replacement remains **not ready** because of distinct public errors, buffered readers, missing native no_std, and incomplete deployment/platform validation. Both strict surfaces remain upstream delegation, and the standalone facade has no native engine dependency. It is a compatibility alias rather than independent acceleration. Existing optimization measurements are historical submitted evidence; benchmarks were not rerun for these key changes. The latest recorded native Twitter serialization at 82.36–83.00 us versus oracle 48.39–48.73 us is about **1.7 times the time**, and Canada does not establish a robust win.

Keep **`serde_json = "=1.0.151"`** visible in facade, core optional dependency and fuzz oracle manifests. Any pin update requires source review of key/byte/private-token and numeric contracts, rerunning native regressions, typed differential fuzz, facade/feature and isolated consumer checks, plus final-head CI/Fuzz; do not silently widen the supported version range. Follow the planned **0.4.0** breaking-change release strategy before publication. Native no_std and streaming readers remain separate substantial work, not prerequisites for honestly merging a scoped repair after its actual checks pass.

Follow-up native optimization implementation commit: **`716b782fb68d97a7d8b505f2e2e246a3f2b411c1`**, with final validation and measurements tracked in [optimization progress](optimization-progress.md). The recorded baseline commits, allocation data and benchmark tables below remain historical evidence for their stated source fingerprints. Current native writers stream; `to_writer_buffered` preserves earlier ordering, while native readers remain buffered. Existing no-std/platform/error-surface migration limits remain open.

## Identity and scope

Starting and pre-commit validation HEAD: `a01708d23ea1d748b5e084fe5b705eb6092d27d8`. **Final implementation commit: `82c496e3c1d2bb3530b837be772f58f2c3a72c60`**, on local branch `repair/replacement-readiness`. This report's commit annotation is a subsequent documentation-only commit; it does not alter the validated production source. Measurements were collected from the uncommitted implementation, so their baseline HEAD and dirty flag remain historical evidence. Exact Rust source fingerprints, lockfile hashes, compiler, CPU and dataset hashes accompany each measurement. Final Rust-source fingerprint: `cf83002380694fcbb069cc246122db70c371f3b74f7f5d8ae3c4509081b05a71`. Measurement metadata predates only the additional enclosing-deserializer panic regression; production Rust sources are unchanged. No reset, push, merge or publication occurred.

Commit readiness was checked again: workspace tests, formatting, source-fingerprint comparison and staged diff checks passed. Three intentionally whitespace-bearing conformance fixtures retain their original bytes; a corpus-local Git attribute disables whitespace diagnostics for those JSON fixtures. No unexpected large generated files were staged. Open release/migration gates below remain open despite local commit readiness.

Oracle: **serde_json 1.0.151**, pinned in the facade and optional core dependency. The resolved registry source (`de.rs`, `read.rs`, serializers and number implementation) and [official API documentation](https://docs.rs/serde_json/1.0.151/serde_json/) were reviewed. `cargo search serde_json --limit 1` confirmed the registry version during this session. See [resolved dependencies](evidence/resolved-dependencies.lock) and [environment](evidence/environment.json). The crates.io ownership API returned HTTP 403: publication ownership is **NOT VERIFIED**. The public GitHub actions query returned zero visible runs; this is not evidence of passing upstream CI.

Host: Linux x86_64, AMD BC-250, Rust/Cargo 1.97.1. Rust **1.71.0** consumer builds and nightly Miri/portable-SIMD checks were actually executed. Compiler details are in the environment evidence. Native and custom-derive crates remain std-dependent. The standalone facade supports tested std consumers and genuine no_std+alloc compilation; the latter was compiled for `thumbv7em-none-eabi`, not merely a std-capable host.

## Work package status

`VERIFIED` below means the stated repair passed the listed finite checks, not universal equivalence. Unexecuted deployment/release gates remain open.

| ID | Status | Implementation, evidence and remaining scope |
|---|---|---|
| S1 | VERIFIED | `scanner.rs` removes the raw-pointer guard. The independent Arc/atomic budget is allocated lazily on first composite entry and may safely outlive its scanner. Error/unwind and owner-drop Miri regressions pass. Checked UTF-8 replaces unchecked conversion; forged `BorrowedNoEsc` is escaped on output; SIMD offset and sink arithmetic repaired. Audit scope is these trust boundaries, not every possible unsafe dependency. |
| C1 | VERIFIED | `serde_impl.rs` explicitly finalizes sequences, tuples, maps, structs and enum wrappers after successful visitors; no discarded fallible Drop validation. Malformed and early-return container regressions pass; visitor errors retain precedence. |
| C2 | VERIFIED | Shared escape/hex validation, allocation-free lexical skip, iterative ignored/raw container validation. Separate borrowed/owned raw, ignored, string and byte model tests pass, including lone-surrogate/non-UTF8 distinctions. JSONTestSuite and bounded differential fuzz pass. |
| C3 | VERIFIED | Unicode scalar char visitor, numeric bounds/callback dispatch, map keys, bytes, transient/borrowed strings, enum/struct representations and default/roundtrip float-bit tests pass. `native_number.rs` independently implements the pinned default significand policy; roundtrip mode is explicit. This does not establish identical native public errors or all arbitrary custom implementations. |
| A1 | VERIFIED | Native and facade unsized serialization compiles without extra references. Renamed facade-only, native, derive, interop and compile-fail fixtures pass. Full upstream facade exports preserve Value/Number/Error identity. Unsupported Mode A attributes now produce diagnostics; supported subset documented. |
| A2 | VERIFIED | Both strict facades select real upstream before invoking user code and never retry. Invocation-count, fresh-state and callback/error tests pass. No native acceleration is claimed for Mode C. |
| A3 | VERIFIED | Strict reader/writer APIs directly re-export upstream streaming operations. Short/interrupted I/O, partial output, callback ordering and error precedence regressions pass. Native readers remain explicitly buffered. Follow-up native writer streaming and callback tests are documented separately in optimization progress; native public errors remain a distinct type. |
| F1 | BLOCKED | Facade std/alloc and mirror features, recursion controls, isolated consumers and MSRV validated. Core/native no_std deployment is unimplemented; host `--no-default-features` builds do not prove it. Windows/macOS/aarch64 runtime gates are configured but NOT RUN locally. These configurations cannot receive a native replacement verdict. |
| P1 | VERIFIED | Prefix-plus-slack decoded-string allocation removes unrelated-suffix amplification. Instrumented baseline and repaired workloads establish linear retained capacity and allocated bytes for increasing short-string arrays; long/plain/late/dense/Unicode tradeoffs measured. |
| P2 | IN PROGRESS | `to_bytes_in` directly reuses the supplied Vec, with documented append/reset/error/unwind behavior; pointer/capacity and partial-output tests pass. Lazy budget removes scalar/borrowed allocation. Component timing and scalar/SWAR/SSE2/host-dispatch/portable-SIMD boundary tests executed. Targeted buffer/allocation improvements are verified; full call-graph profiling is NOT RUN because perf/valgrind are unavailable, and untested hardware dispatch remains open. |
| B1 | VERIFIED | Every serialization denominator uses its own emitted bytes; preflight correctness checks execute. Projected/full Value and timed-copy/fresh/reuse contracts labelled. Actual facade benchmark added. Workflow/report generator isolates current invocation from archived history. Executed measurements and uncertainty accompany this report. |
| Q1 | IMPLEMENTED / NOT VALIDATED | Deterministic differential/conformance tests, isolated consumers, compile failures, five fuzz targets, Miri, MSRV and local release checks pass. CI jobs cover real arm runner, Windows/macOS, nightly SIMD and scheduled fuzz; those remote jobs and long fuzz campaigns have NOT RUN. Release-wide platform gates remain open. |
| D1 | VERIFIED | Root/crate docs, methodology, changelog, attribution, feature/migration limits and package repository metadata updated. Prior unsupported speed headlines withdrawn and archived separately. No release published. |

## Reproduction and safety design

Four source-review findings were reproduced as failing baseline tests before repair: extra tuple elements, multi-character char, ignored `\\q`, and tiny escaped-string capacity proportional to the unrelated suffix (100007 bytes). The historical boxed-scanner owner-drop reproducer ran **only under Miri**, which reported a dangling memory access and exited 1. Baseline logs and isolated checkout are outside production artifacts in `/tmp/jzon-readiness-evidence` and `/tmp/jzon-readiness-baseline`.

The new guard owns the depth counter independently. It contains no scanner pointer or scanner reference, so dropping/moving the scanner cannot invalidate its destructor. Public fields cannot forge the counter. Borrowed outputs still borrow the input lifetime. On success, error or unwind the guard returns its budget; catching a visitor panic does not leak depth. An additional native visitor closes its first array and then panics; the same deserializer subsequently parses a 127-level next value successfully under Miri. Input position is retained rather than rolled back. Scanner clones intentionally share the same budget, which can conservatively couple active clone nesting. Retaining a guard consumes budget until drop, but cannot cause an invalid memory access. A guard-move compile-fail test is unnecessary for this ownership model; the formerly invalid owner-drop program is now sound and is exercised under Miri.

Default typed nesting permits 127 opens and rejects the next, matching the pinned reference boundary tests. Merely enabling `unbounded_depth` preserves the default limit; `disable_recursion_limit()` is explicit. Upstream ignored/raw skipping uses an iterative unbounded lexical path, which the native implementation now follows independently. Mode A retains its own bounded recursive policy and documented leniency. Deep unbounded typed deserialization remains the caller's stack-safety responsibility.

Lexical skip is deliberately distinct from UTF-8 string materialization: upstream IgnoredAny/RawValue can accept bytes or surrogate escapes that String rejects. Byte visitors also have their own WTF-8/control-byte contract. Tests assert these separately. No skipped slice is passed to unchecked UTF-8 conversion.

## Executed validation

[Machine-readable command evidence](evidence/validation-results.json) records command, exit status, elapsed time and log location. Logs live under `/tmp/jzon-readiness-evidence/validated`. Earlier failing baseline and development checks are retained separately, not represented as final successes. Final commands below all exited **0**:

- `cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo doc --workspace --no-deps` (workspace tests include doctests).
- Core tests with `compat`; `compat,raw_value,preserve_order,arbitrary_precision,float_roundtrip`; `compat,unbounded_depth`; and `compat,simd,simd-intrinsics,stats,zmij-float-ser`.
- `cargo test -p jzon-rs-compat`; `cargo test -p jzon-rs-serde`; `cargo check -p jzon-rs-serde --no-default-features`; facade no-default alloc check.
- `python3 tools/check-consumers.py`: standalone renamed facade with no oracle dependency, native, derive, alloc, interop/feature-unification and expected compile-error probes. `cargo check --manifest-path fixtures/alloc/Cargo.toml --locked --target thumbv7em-none-eabi`.
- `cargo +1.71.0 check --manifest-path fixtures/{facade,native,alloc,derive}/Cargo.toml --locked`, each invoked separately. This proves selected library consumers at declared MSRV, not all development tools.
- `cargo +nightly miri test -p jzon-rs --test depth_safety --test readiness`, bounded scalar configuration; `cargo +nightly test --workspace --features simd,unstable` separately.
- `cargo bench -p jzon-rs --bench bench_cmp --features float_roundtrip -- --test`, all benchmark preflight checks.
- `cargo audit --json`; `cargo package --workspace --allow-dirty`. Workspace staging validates interdependent unpublished packages; an earlier isolated native package attempt against the published 0.3.0 dependency failed and was not hidden. No publish command was executed.
- Five nightly fuzz targets (`native`, `skip`, `raw`, `roundtrip`, `simd`), both default and `roundtrip_policy`: each uses `-max_total_time=5 -max_len=4096 -rss_limit_mb=512`. These are smoke campaigns. Fuzzing discovered the eight-control-byte whitespace bug, minimized into a deterministic regression before the final rerun.

316 JSONTestSuite cases up to 4096 bytes are vendored with hashes, source commit and MIT license in `crates/jzon/tests/corpus`. Two oversized resource cases are explicitly excluded. `y_`/`n_` expectations are asserted against the reference, and implementation-defined `i_` cases compared to the pinned oracle rather than assigned unconditional validity. Large/deep deterministic regressions supplement this bounded corpus.

**NOT RUN:** Windows/macOS/aarch64 runtime execution (no such local execution environment); remote GitHub jobs (no push authorized); long scheduled fuzz campaigns (only bounded smoke runs executed); every possible CPU/feature/consumer combination; publication ownership verification (API access denied). Intrinsics were tested on the host plus explicit scalar/SSE2 paths; this does not emulate all unsupported-CPU dispatch cases. Native no_std is an actual missing capability, not merely an unavailable toolchain. No unqualified all-platform readiness declaration is made.

## Allocation and performance

See `evidence/allocation.csv`, `allocation-baseline.csv` and their RSS evidence. For 100, 1000 and 10000 short escaped strings, native retained decoded capacities fell from **25150 / 2501500 / 250015000** bytes to **100 / 1000 / 10000**. Mode A retains 16 bytes per short decoded string. Native allocation count is approximately 2n+2 because transient Serde strings may be copied by the destination; the baseline used a different callback ownership path. These allocations and vector growth are linear, while historical retained capacity was quadratic. Scalar and borrowed-string parsing now reports **zero allocations** in the isolated probe.

RSS is a process high-water measurement, distinct from retained virtual capacity; allocator demand paging means 250 MB of reserved capacity is not 250 MB resident. The repaired probe recorded 12600 KiB peak RSS versus 43400 KiB at baseline. Single-sample times in allocation CSV are diagnostic only. Long late/dense/Unicode strings trade extra growth or transient copies for bounded retained storage; no claim that every string workload became faster is made.

Criterion measurements report mean ns/op, 95% intervals, emitted/input bytes, compiler/features/allocator and dataset hashes. Review threshold was declared before interpretation: repeatable >=5% improvement and non-overlapping intervals for equivalent contracts. Shared-host exploratory results cannot justify precise deployment guarantees. Baseline component measurements use an isolated old checkout with the same harness; its unavailable direct-reuse operation is labelled `prior_buffered_path`, not misrepresented as reuse. Historical results never replace current estimates.

Executed tables: [Mode A/B Twitter and Canada](evidence/native-results.md), [actual Mode C](evidence/compat-results.md), [native components](evidence/native-component-results.md), [baseline components](evidence/native-baseline-component-results.md), and [intermediate eager-counter components](evidence/pre-lazy-native-component-results.md). Raw estimates and invocation metadata sit beside each table; [benchmark commands](evidence/benchmark-commands.json) record actual exits. Mode C delegates to the exact engine against which it is measured, so sampling differences are not independent-engine acceleration. Native buffered writer and direct buffer reuse are separate cases; projected Twitter parsing cannot be compared to full Value materialization.

Current primary invocation (float_roundtrip, 200 samples, 1s warmup/4s measurement) serialized the identical 51013-byte projected Twitter output in 54.03 us (A), 93.40 us (B), versus 47.62 us (reference). Canada output was 2090326 bytes: 4.359 ms (A), 3.777 ms (B), 3.316 ms (reference). These are regressions relative to the reference, not acceleration evidence. The full Value facade workload uses a different 631514-byte input and 466906-byte output and cannot share those denominators.

The first component run measured direct native reuse at 286.68 ns [280.78,292.75] versus corrected buffered writer 325.26 ns [311.48,340.96]; this isolated sample is insufficient for a broad or repeatable speed claim. Scalar/borrowed allocation removal is independently measured by allocation instrumentation. Component baseline correctness and callback contracts differ, so elapsed-time changes do not erase the safety repair. Repeated current invocations are retained separately to expose shared-host variation, never substituted with historical best values: [native repeat](evidence/native-repeat-results.md), [facade repeat](evidence/compat-repeat-results.md), [component repeat](evidence/profile-repeat-results.md). The repeat again showed B slower than the reference: Twitter 90.73 us versus 48.24 us; Canada 3.734 ms versus 3.335 ms. Facade fresh output ranged 641.5–664.5 us while reference ranged 660.2–972.4 us across these invocations; direct re-export means these differences cannot be attributed to an independent engine. Writer samples are similarly noisy. The extra visitor-unwind Miri check briefly overlapped the tail of the repeat facade campaign, another reason to reject precise speed conclusions. Primary campaigns were run sequentially after other heavy validation completed.

Native raw/number integration recognizes upstream private `$serde_json::private::{RawValue,Number}` tokens. This is an explicit internal-protocol dependency, tested only against pinned **1.0.151**; a wider upstream version range is not advertised. Downstream upstream-only feature activation is tested through the standalone facade interop fixture; native features still require explicit configuration. `cargo tree -p jzon-rs-serde -e normal` shows no real serde_json production dependency for the native-only wrapper; `cargo tree -p jzon-rs-compat -e normal` does show it.

## Migration and separate verdicts

| Question | Assessment |
|---|---|
| Safety findings repaired? | **Yes for the reviewed guard, UTF-8, forgeable string, SIMD offset and capacity boundaries**, supported by regressions and executed Miri. This is a scoped audit, not a proof about all dependencies. |
| Native engine validated for its claimed contract? | **Substantially validated for documented std typed/Serde behavior**, default and roundtrip numerical policies, native container/skip/visitor tests and conformance corpus. It is **not approved as unrestricted serde_json replacement**: native errors and buffered reader I/O differ, no_std is missing, and deployment gates remain open. |
| Mode C suitable? | **Suitable for the tested pinned-1.0.151 std consumer scope**, with exact upstream types, callbacks, errors and streaming delegated once; no_std+alloc compile scope also tested. Other platforms require their configured runtime gates. |
| Measured workload faster? | **No broad speed verdict.** Use the mode-specific tables; safety/visitor repairs incur some regressions, allocation amplification is repaired, and Mode C is delegated. |
| Real serde_json removed? | **No.** Both strict facades re-export/delegate upstream; the standalone facade directly depends on it. Native Mode B does not route operations through it, but oracle dependencies remain for validation and optional compatibility. |

Breaking changes: `DepthGuard` no longer has its historical lifetime generic; migrate explicitly named types accordingly. Stats names now describe scanner events (`decoded_strings`, `number_bytes_scanned`) rather than claiming total allocator activity. Feature availability no longer disables recursion automatically. Unsupported derive attributes that were silently ignored now fail compilation. Standalone facade performance-named flags are documented no-ops, and native default float formatting uses zmij matching the pinned reference. Recommend **0.4.0** for these 0.x API changes; version was not changed or published.

A dependency rename to the standalone facade preserves the tested upstream contract because it retains upstream itself. Choosing native APIs is a separate migration decision requiring the documented error/I/O/std/numeric contract. Removing upstream entirely, implementing native streaming readers/no_std, executing remote platform jobs and longer fuzz campaigns remain open work; this report does not mark the complete specification universally ready.
