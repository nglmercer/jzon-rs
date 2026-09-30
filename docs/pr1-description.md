# Implement native readers/alloc support and audit facade safety for planned 0.4.0

The pinned upstream boolean-key parser could enter an invalid UTF-8 path on a
non-ASCII string key. Both strict facades now select the audited local 1.0.151
source, whose narrow fix preserves the initial byte until literal dispatch.
The affected original reproducer fails under Miri; both patched facades, public
constructors/streams and isolated source/package consumers are checked under Miri.
Package metadata also proves that consumers without a root safe-source override
resolve the unsafe registry copy. Registry-only publication remains blocked; no
fixed upstream release is assumed or invented.

Native Serde now supplies error categories and byte-based input positions,
maintains visitor-error precedence, and parses Read inputs incrementally through
native code. Reader/string/slice streams validate primitive token boundaries.
Fragmentation, Interrupted I/O, EOF, partial consumption, container completion,
custom key/byte visitors and unwind depth restoration have regression tests.
Borrowed scalar callbacks become transient callbacks on readers; callbacks are
never replayed. Explicit buffered helpers retain the former contract.

Core and native wrapper support no_std + alloc on atomic-capable targets. Std/I/O
and HashMap APIs are gated, derive output uses core/alloc paths, and weak mirror
forwarding avoids activating serde_json for native consumers. All internal
packages are aligned to planned 0.4.0. Existing map-key, guard, allocation and
container repairs are retained. See [migration notes](migration-0.4.md).

Local validation includes formatting, Clippy with warnings denied, complete
workspace/mirror/SIMD/strict suites, nightly SIMD, isolated consumers, Rust 1.71,
real thumbv7em-none-eabi native/derive/facade alloc checks, targeted Miri, typed and
Value reader differential fuzzing, full package build checks and extracted
package source/type verification. Exact results, fingerprints and finite
coverage limits are in [replacement readiness](replacement-readiness.md).

Repeated full-reader measurements show incremental native taking approximately
3.2–3.6 times the reference time for a 10,000-u32 array. Separate allocation counts
show removal of the whole-document input allocation, while owned output dominates
memory. No unmeasured acceleration or universal replacement claim is made.
CPU-callgraph profiling tools are unavailable; timing optimization remains work
requiring profiling. Shared-host timing variation is recorded.

Compatibility facades remain upstream delegation. Native-only JSON dependency
removal, native completeness, compatibility, measured acceleration and registry
release readiness have separate conclusions. Exact upstream error/stream recovery
contracts, unexecuted platforms/deployment workloads and safe registry distribution
remain limitations. Remote workflows were excluded by the user; no job, push,
merge or publication was performed.
