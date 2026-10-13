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
| mode_b/component_parse/float | 43.73 | 43.47–44.05 | 22 | 479.73 |
| mode_b/component_parse/integer | 102.71 | 101.69–103.91 | 20 | 185.71 |
| mode_b/component_parse/string/borrowed | 28.80 | 28.62–29.01 | 18 | 596.00 |
| mode_b/component_parse/string/dense | 77.93 | 77.26–78.81 | 20 | 244.76 |
| mode_b/component_parse/string/late_escape | 58.28 | 57.17–59.73 | 19 | 310.92 |
| mode_b/component_parse/string/plain | 48.83 | 48.02–49.71 | 18 | 351.55 |
| mode_b/component_parse/string/unicode | 106.61 | 105.77–107.69 | 26 | 232.57 |
| mode_b/component_parse/wide/field_dispatch | 904.68 | 901.98–907.50 | 104 | 109.63 |
| mode_b/component_serialize/floats/fresh | 4069.62 | 3894.95–4272.58 | 2115 | 495.63 |
| mode_b/component_serialize/integers/fresh | 1653.72 | 1650.20–1657.63 | 1546 | 891.55 |
| mode_b/component_serialize/string/escaped | 33.46 | 33.27–33.63 | 15 | 427.56 |
| mode_b/component_serialize/string/plain | 27.78 | 27.54–28.22 | 18 | 617.85 |
| mode_b/component_serialize/wide/buffered_writer | 291.17 | 288.50–294.28 | 104 | 340.63 |
| mode_b/component_serialize/wide/direct_reuse | 273.64 | 271.51–276.57 | 104 | 362.46 |
| mode_b/component_serialize/wide/fresh | 291.63 | 289.29–294.53 | 104 | 340.09 |
| mode_b/depth/native/1 | 91.48 | 90.33–93.19 | 6 | 62.55 |
| mode_b/depth/native/127 | 8874.70 | 8854.42–8902.83 | 258 | 27.72 |
| mode_b/depth/native/16 | 773.21 | 765.85–781.69 | 36 | 44.40 |
| mode_b/depth/native/64 | 4085.85 | 4067.02–4108.96 | 132 | 30.81 |
| mode_b/depth/public_guard | 22.16 | 22.06–22.30 | — | — |
