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
| mode_b/component_parse/float | 43.63 | 43.30–44.18 | 22 | 480.83 |
| mode_b/component_parse/integer | 100.76 | 100.41–101.05 | 20 | 189.30 |
| mode_b/component_parse/string/borrowed | 29.01 | 28.75–29.31 | 18 | 591.76 |
| mode_b/component_parse/string/dense | 79.86 | 78.52–81.51 | 20 | 238.85 |
| mode_b/component_parse/string/late_escape | 58.41 | 58.17–58.66 | 19 | 310.23 |
| mode_b/component_parse/string/plain | 46.73 | 46.31–47.15 | 18 | 367.32 |
| mode_b/component_parse/string/unicode | 109.78 | 108.36–111.34 | 26 | 225.87 |
| mode_b/component_parse/wide/field_dispatch | 906.94 | 900.41–915.15 | 104 | 109.36 |
| mode_b/component_serialize/floats/fresh | 3760.73 | 3736.64–3790.51 | 2115 | 536.34 |
| mode_b/component_serialize/integers/fresh | 1729.87 | 1709.11–1755.88 | 1546 | 852.31 |
| mode_b/component_serialize/string/escaped | 33.76 | 32.87–34.93 | 15 | 423.73 |
| mode_b/component_serialize/string/plain | 28.85 | 27.81–30.37 | 18 | 594.98 |
| mode_b/component_serialize/wide/buffered_writer | 297.55 | 295.75–300.21 | 104 | 333.33 |
| mode_b/component_serialize/wide/direct_reuse | 304.00 | 286.34–325.61 | 104 | 326.26 |
| mode_b/component_serialize/wide/fresh | 289.27 | 285.24–294.61 | 104 | 342.86 |
| mode_b/depth/native/1 | 88.26 | 85.91–91.24 | 6 | 64.84 |
| mode_b/depth/native/127 | 8828.51 | 8782.15–8890.72 | 258 | 27.87 |
| mode_b/depth/native/16 | 760.66 | 751.55–771.35 | 36 | 45.13 |
| mode_b/depth/native/64 | 4060.36 | 4020.56–4109.16 | 132 | 31.00 |
| mode_b/depth/public_guard | 22.77 | 22.74–22.81 | — | — |
