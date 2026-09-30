# Unreleased — recommend 0.4.0

- Remove raw-pointer depth restoration. DepthGuard owns counter storage, no longer has a lifetime parameter, and survives scanner movement/destruction safely. The first composite parse lazily allocates this state; scalar parsing stays allocation-free.
- Default depth limit remains active with unbounded_depth; explicit disabling is required. Guard cleanup restores depth on errors and panic unwinding.
- Finalize native Serde containers explicitly; reject extra tuple elements and malformed tails. Validate ignored escapes and control bytes. Correct bulk whitespace acceptance.
- Match scalar visitor dispatch, borrowed/transient string callbacks, byte-string rules, map-key grammar and struct sequence representations. Native float parsing follows default/reference or float_roundtrip policy; float serialization uses zmij.
- Accept unsized serialization arguments. Add native to_bytes_in for direct buffer reuse with append and partial-error semantics.
- Strict facades re-export serde_json once; remove callback replay and buffered facade I/O. Standalone facade supports real no_std + alloc and pins upstream 1.0.151; performance flags there are legacy no-ops.
- Rename misleading ScannerStats fields to decoded_strings and number_bytes_scanned. Prefix-sized escaped-string allocations replace document-sized capacity.
- Reject unsupported Mode A attributes; expand native regressions, consumers, safety/fuzz/platform CI and mode-specific benchmark reporting. Withdraw historical headline speed claims.

No package has been published, branch pushed, or release merged. See docs/replacement-readiness.md for exact validation limits.
