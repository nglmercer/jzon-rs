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
  "source_fingerprint": "ef7d2ce6ca6543f0dc52c68122920d09464c922d972d4dd946c0ea9509e31321"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 43.29 | 43.17–43.40 | 22 | 484.69 |
| mode_b/component_parse/integer | 104.43 | 100.67–109.37 | 20 | 182.65 |
| mode_b/component_parse/string/borrowed | 28.40 | 28.29–28.53 | 18 | 604.48 |
| mode_b/component_parse/string/dense | 75.53 | 75.08–76.03 | 20 | 252.53 |
| mode_b/component_parse/string/late_escape | 58.03 | 57.45–58.82 | 19 | 312.25 |
| mode_b/component_parse/string/plain | 45.25 | 44.95–45.57 | 18 | 379.37 |
| mode_b/component_parse/string/unicode | 105.77 | 105.20–106.73 | 26 | 234.42 |
| mode_b/component_parse/wide/field_dispatch | 893.46 | 892.59–894.51 | 104 | 111.01 |
| mode_b/component_serialize/floats/fresh | 3738.18 | 3725.06–3756.17 | 2115 | 539.57 |
| mode_b/component_serialize/integers/fresh | 1664.37 | 1659.52–1670.22 | 1546 | 885.85 |
| mode_b/component_serialize/string/escaped | 33.62 | 33.54–33.72 | 15 | 425.56 |
| mode_b/component_serialize/string/plain | 27.48 | 27.42–27.54 | 18 | 624.77 |
| mode_b/component_serialize/wide/buffered_writer | 285.69 | 282.94–289.94 | 104 | 347.17 |
| mode_b/component_serialize/wide/direct_reuse | 277.67 | 276.74–278.95 | 104 | 357.19 |
| mode_b/component_serialize/wide/fresh | 282.93 | 282.44–283.41 | 104 | 350.55 |
| mode_b/depth/native/1 | 90.93 | 90.43–91.71 | 6 | 62.93 |
| mode_b/depth/native/127 | 9161.70 | 9103.59–9256.07 | 258 | 26.86 |
| mode_b/depth/native/16 | 749.32 | 748.10–750.55 | 36 | 45.82 |
| mode_b/depth/native/64 | 4128.70 | 4123.76–4134.04 | 132 | 30.49 |
| mode_b/depth/public_guard | 22.69 | 22.68–22.70 | — | — |
