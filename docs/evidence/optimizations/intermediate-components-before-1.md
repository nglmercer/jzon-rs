# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "94727f756af4db62c6ce5892ea4b650d098622be",
  "dirty": true,
  "mode": "A/B optimization components-before-1",
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
| mode_b/component_parse/float | 44.71 | 44.06–45.53 | 22 | 469.28 |
| mode_b/component_parse/integer | 102.26 | 100.29–104.96 | 20 | 186.52 |
| mode_b/component_parse/string/borrowed | 29.41 | 28.52–30.73 | 18 | 583.77 |
| mode_b/component_parse/string/dense | 75.09 | 74.78–75.44 | 20 | 254.01 |
| mode_b/component_parse/string/late_escape | 57.07 | 56.71–57.46 | 19 | 317.52 |
| mode_b/component_parse/string/plain | 44.59 | 44.27–44.93 | 18 | 384.98 |
| mode_b/component_parse/string/unicode | 111.54 | 109.92–113.37 | 26 | 222.31 |
| mode_b/component_parse/wide/field_dispatch | 904.99 | 902.00–908.60 | 104 | 109.59 |
| mode_b/component_serialize/floats/fresh | 3734.21 | 3687.56–3795.75 | 2115 | 540.15 |
| mode_b/component_serialize/integers/fresh | 1691.57 | 1664.06–1727.57 | 1546 | 871.61 |
| mode_b/component_serialize/string/escaped | 35.95 | 34.52–37.54 | 15 | 397.94 |
| mode_b/component_serialize/string/plain | 28.00 | 27.76–28.26 | 18 | 613.12 |
| mode_b/component_serialize/wide/buffered_writer | 302.45 | 293.37–312.56 | 104 | 327.93 |
| mode_b/component_serialize/wide/direct_reuse | 333.84 | 317.80–350.85 | 104 | 297.09 |
| mode_b/component_serialize/wide/fresh | 315.33 | 302.43–330.62 | 104 | 314.53 |
| mode_b/depth/native/1 | 96.52 | 94.25–99.31 | 6 | 59.28 |
| mode_b/depth/native/127 | 9857.60 | 9516.88–10251.62 | 258 | 24.96 |
| mode_b/depth/native/16 | 764.59 | 754.82–776.33 | 36 | 44.90 |
| mode_b/depth/native/64 | 4114.96 | 4062.01–4180.21 | 132 | 30.59 |
| mode_b/depth/public_guard | 22.76 | 22.71–22.81 | — | — |
