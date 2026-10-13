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
  "source_fingerprint": "45b17eeff552ab63e5dfef7fc0535fe9e90d8a9389f1b84c84c1d84993619e53"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 43.92 | 43.26–44.74 | 22 | 477.73 |
| mode_b/component_parse/integer | 109.00 | 105.30–113.14 | 20 | 174.99 |
| mode_b/component_parse/string/borrowed | 28.66 | 28.53–28.80 | 18 | 598.94 |
| mode_b/component_parse/string/dense | 82.45 | 81.62–83.57 | 20 | 231.33 |
| mode_b/component_parse/string/late_escape | 68.66 | 67.23–70.38 | 19 | 263.92 |
| mode_b/component_parse/string/plain | 53.77 | 51.51–56.20 | 18 | 319.23 |
| mode_b/component_parse/string/unicode | 108.45 | 107.04–110.37 | 26 | 228.63 |
| mode_b/component_parse/wide/field_dispatch | 755.73 | 748.66–764.44 | 104 | 131.24 |
| mode_b/component_serialize/floats/fresh | 3659.07 | 3654.15–3664.33 | 2115 | 551.24 |
| mode_b/component_serialize/integers/fresh | 1682.57 | 1657.23–1717.56 | 1546 | 876.27 |
| mode_b/component_serialize/string/escaped | 35.52 | 34.51–36.67 | 15 | 402.68 |
| mode_b/component_serialize/string/plain | 28.38 | 28.21–28.57 | 18 | 604.91 |
| mode_b/component_serialize/wide/buffered_writer | 341.25 | 334.88–348.65 | 104 | 290.64 |
| mode_b/component_serialize/wide/direct_reuse | 342.76 | 323.24–365.31 | 104 | 289.37 |
| mode_b/component_serialize/wide/fresh | 370.65 | 355.40–386.11 | 104 | 267.59 |
| mode_b/component_serialize/wide/reusable_serializer | 355.90 | 345.78–366.59 | 104 | 278.68 |
| mode_b/component_serialize/wide/streaming_to_vec | 290.63 | 283.09–298.91 | 104 | 341.26 |
| mode_b/depth/native/1 | 84.34 | 82.84–86.10 | 6 | 67.85 |
| mode_b/depth/native/127 | 10638.08 | 10198.43–11084.71 | 258 | 23.13 |
| mode_b/depth/native/16 | 741.51 | 705.78–781.61 | 36 | 46.30 |
| mode_b/depth/native/64 | 4451.56 | 4342.58–4570.96 | 132 | 28.28 |
| mode_b/depth/public_guard | 22.04 | 21.92–22.18 | — | — |
