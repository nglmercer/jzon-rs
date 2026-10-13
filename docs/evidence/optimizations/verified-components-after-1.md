# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "94727f756af4db62c6ce5892ea4b650d098622be",
  "dirty": true,
  "mode": "A/B optimization components-after-1",
  "features": "float_roundtrip",
  "cpu": "AMD BC-250",
  "os": "Linux-7.2.8-1-cachyos-x86_64-with-glibc2.44",
  "compiler": "rustc 1.97.1 (8bab26f4f 2026-07-14)\nbinary: rustc\ncommit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452\ncommit-date: 2026-07-14\nhost: x86_64-unknown-linux-gnu\nrelease: 1.97.1\nLLVM version: 22.1.6\n",
  "cargo": "cargo 1.97.1 (c980f4866 2026-06-30)",
  "rustflags": "",
  "allocator": "system default (allocation probe uses stats_alloc instrumentation)",
  "lockfile_sha256": "6ecd1dfab7853bfc73c94d874b670db20b0c9329bf3e2434b7cd88e11308a8ca",
  "datasets": {
    "crates/jzon/data/canada.json": "f83b3b354030d5dd58740c68ac4fecef64cb730a0d12a90362a7f23077f50d78",
    "crates/jzon/data/citm_catalog.json": "a73e7a883f6ea8de113dff59702975e60119b4b58d451d518a929f31c92e2059",
    "crates/jzon/data/generated_50k.json": "d4d43218e6946c0966fe532a7da0f792a7480ef7606751ffbed2a07f8f47eb56",
    "crates/jzon/data/mixed_2mb.json": "2da7e3b9e31d7669356f7b4216095fddaa0644c1a327dbe2c328e78e3710183c",
    "crates/jzon/data/twitter.json": "a08b769f32b95f426cbc3abafcec65c1a19d3eb544d4ddf320eae142c99efc5d"
  },
  "sample_settings": "30 samples/.2s warmup/.5s measurement",
  "source_fingerprint": "8e17cb53d13682e0eb12a6544c020ba9c64bc96be17b1d66adb18f1e6cadf6aa"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 43.19 | 43.16–43.23 | 22 | 485.74 |
| mode_b/component_parse/integer | 100.29 | 99.89–100.67 | 20 | 190.19 |
| mode_b/component_parse/string/borrowed | 27.88 | 27.68–28.16 | 18 | 615.78 |
| mode_b/component_parse/string/dense | 85.35 | 85.11–85.59 | 20 | 223.48 |
| mode_b/component_parse/string/late_escape | 62.83 | 62.69–62.97 | 19 | 288.40 |
| mode_b/component_parse/string/plain | 40.39 | 40.13–40.76 | 18 | 425.00 |
| mode_b/component_parse/string/unicode | 110.17 | 109.67–110.75 | 26 | 225.06 |
| mode_b/component_parse/wide/field_dispatch | 716.93 | 715.32–719.28 | 104 | 138.34 |
| mode_b/component_serialize/floats/fresh | 3681.93 | 3676.88–3687.33 | 2115 | 547.82 |
| mode_b/component_serialize/integers/fresh | 1595.13 | 1589.26–1601.29 | 1546 | 924.30 |
| mode_b/component_serialize/string/escaped | 32.92 | 32.50–33.33 | 15 | 434.53 |
| mode_b/component_serialize/string/plain | 24.54 | 24.52–24.57 | 18 | 699.50 |
| mode_b/component_serialize/wide/buffered_writer | 171.67 | 169.20–174.86 | 104 | 577.74 |
| mode_b/component_serialize/wide/direct_reuse | 152.83 | 152.51–153.20 | 104 | 648.99 |
| mode_b/component_serialize/wide/fresh | 164.94 | 164.65–165.25 | 104 | 601.34 |
| mode_b/component_serialize/wide/reusable_serializer | 155.13 | 149.43–162.67 | 104 | 639.35 |
| mode_b/component_serialize/wide/streaming_to_vec | 229.90 | 229.11–231.23 | 104 | 431.42 |
| mode_b/depth/native/1 | 73.05 | 73.00–73.10 | 6 | 78.33 |
| mode_b/depth/native/127 | 8430.49 | 8354.33–8531.23 | 258 | 29.19 |
| mode_b/depth/native/16 | 645.43 | 634.16–662.72 | 36 | 53.19 |
| mode_b/depth/native/64 | 3783.99 | 3767.83–3805.02 | 132 | 33.27 |
| mode_b/depth/public_guard | 22.67 | 22.66–22.67 | — | — |
