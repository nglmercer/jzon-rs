# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "94727f756af4db62c6ce5892ea4b650d098622be",
  "dirty": true,
  "mode": "A/B optimization components-before-2",
  "features": "float_roundtrip",
  "cpu": "AMD BC-250",
  "os": "Linux-7.2.8-1-cachyos-x86_64-with-glibc2.44",
  "compiler": "rustc 1.97.1 (8bab26f4f 2026-07-14)\nbinary: rustc\ncommit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452\ncommit-date: 2026-07-14\nhost: x86_64-unknown-linux-gnu\nrelease: 1.97.1\nLLVM version: 22.1.6\n",
  "cargo": "cargo 1.97.1 (c980f4866 2026-06-30)",
  "rustflags": "",
  "allocator": "system default (allocation probe uses stats_alloc instrumentation)",
  "lockfile_sha256": "6ecd1dfab7853bfc73c94d874b670db20b0c9329bf3e2434b7cd88e11308a8ca",
  "datasets": {
    "crates/jzon/data/twitter.json": "a08b769f32b95f426cbc3abafcec65c1a19d3eb544d4ddf320eae142c99efc5d",
    "crates/jzon/data/mixed_2mb.json": "2da7e3b9e31d7669356f7b4216095fddaa0644c1a327dbe2c328e78e3710183c",
    "crates/jzon/data/generated_50k.json": "d4d43218e6946c0966fe532a7da0f792a7480ef7606751ffbed2a07f8f47eb56",
    "crates/jzon/data/citm_catalog.json": "a73e7a883f6ea8de113dff59702975e60119b4b58d451d518a929f31c92e2059",
    "crates/jzon/data/canada.json": "f83b3b354030d5dd58740c68ac4fecef64cb730a0d12a90362a7f23077f50d78"
  },
  "sample_settings": "30 samples/.2s warmup/.5s measurement",
  "source_fingerprint": "b609a81d383da50e2c3f11670fbac750f0f40eb508956b5ee96604c327de59b3"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 51.75 | 49.42–54.20 | 22 | 405.39 |
| mode_b/component_parse/integer | 108.16 | 105.20–111.68 | 20 | 176.34 |
| mode_b/component_parse/string/borrowed | 35.29 | 33.16–37.56 | 18 | 486.49 |
| mode_b/component_parse/string/dense | 90.65 | 87.69–93.65 | 20 | 210.40 |
| mode_b/component_parse/string/late_escape | 69.58 | 66.42–72.95 | 19 | 260.42 |
| mode_b/component_parse/string/plain | 53.78 | 51.42–56.41 | 18 | 319.21 |
| mode_b/component_parse/string/unicode | 120.31 | 115.83–125.18 | 26 | 206.09 |
| mode_b/component_parse/wide/field_dispatch | 1114.74 | 1097.95–1132.57 | 104 | 88.97 |
| mode_b/component_serialize/floats/fresh | 4025.10 | 3941.01–4124.52 | 2115 | 501.11 |
| mode_b/component_serialize/integers/fresh | 1797.72 | 1759.27–1840.43 | 1546 | 820.14 |
| mode_b/component_serialize/string/escaped | 38.99 | 37.56–40.57 | 15 | 366.92 |
| mode_b/component_serialize/string/plain | 29.18 | 28.18–30.34 | 18 | 588.29 |
| mode_b/component_serialize/wide/buffered_writer | 297.38 | 290.66–305.23 | 104 | 333.52 |
| mode_b/component_serialize/wide/direct_reuse | 306.47 | 295.49–318.46 | 104 | 323.63 |
| mode_b/component_serialize/wide/fresh | 284.36 | 281.45–287.99 | 104 | 348.79 |
| mode_b/depth/native/1 | 94.19 | 92.39–96.68 | 6 | 60.75 |
| mode_b/depth/native/127 | 9830.30 | 9609.56–10073.37 | 258 | 25.03 |
| mode_b/depth/native/16 | 800.60 | 784.99–818.77 | 36 | 42.88 |
| mode_b/depth/native/64 | 4455.19 | 4347.17–4587.91 | 132 | 28.26 |
| mode_b/depth/public_guard | 22.11 | 22.05–22.17 | — | — |
