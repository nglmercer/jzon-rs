# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "94727f756af4db62c6ce5892ea4b650d098622be",
  "dirty": true,
  "mode": "A/B optimization components-after-2",
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
| mode_b/component_parse/float | 44.41 | 43.66–45.29 | 22 | 472.44 |
| mode_b/component_parse/integer | 108.32 | 105.14–112.44 | 20 | 176.08 |
| mode_b/component_parse/string/borrowed | 31.54 | 30.69–32.50 | 18 | 544.23 |
| mode_b/component_parse/string/dense | 84.48 | 83.77–85.28 | 20 | 225.76 |
| mode_b/component_parse/string/late_escape | 67.06 | 66.30–67.86 | 19 | 270.20 |
| mode_b/component_parse/string/plain | 48.60 | 47.73–49.64 | 18 | 353.19 |
| mode_b/component_parse/string/unicode | 110.51 | 108.92–112.29 | 26 | 224.37 |
| mode_b/component_parse/wide/field_dispatch | 805.50 | 786.79–826.17 | 104 | 123.13 |
| mode_b/component_serialize/floats/fresh | 3970.28 | 3895.27–4052.10 | 2115 | 508.03 |
| mode_b/component_serialize/integers/fresh | 1818.85 | 1744.62–1901.94 | 1546 | 810.61 |
| mode_b/component_serialize/string/escaped | 37.69 | 36.59–38.99 | 15 | 379.56 |
| mode_b/component_serialize/string/plain | 31.74 | 30.64–32.94 | 18 | 540.88 |
| mode_b/component_serialize/wide/buffered_writer | 348.69 | 338.32–361.30 | 104 | 284.44 |
| mode_b/component_serialize/wide/direct_reuse | 321.81 | 315.38–329.15 | 104 | 308.20 |
| mode_b/component_serialize/wide/fresh | 335.71 | 329.14–342.92 | 104 | 295.44 |
| mode_b/component_serialize/wide/reusable_serializer | 353.36 | 337.33–371.76 | 104 | 280.68 |
| mode_b/component_serialize/wide/streaming_to_vec | 284.26 | 277.02–291.73 | 104 | 348.92 |
| mode_b/depth/native/1 | 91.63 | 89.59–93.80 | 6 | 62.45 |
| mode_b/depth/native/127 | 9316.28 | 9073.05–9575.68 | 258 | 26.41 |
| mode_b/depth/native/16 | 704.15 | 688.10–722.57 | 36 | 48.76 |
| mode_b/depth/native/64 | 4454.17 | 4345.41–4598.96 | 132 | 28.26 |
| mode_b/depth/public_guard | 21.87 | 21.84–21.91 | — | — |
