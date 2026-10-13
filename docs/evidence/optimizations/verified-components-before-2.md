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
  "source_fingerprint": "ef7d2ce6ca6543f0dc52c68122920d09464c922d972d4dd946c0ea9509e31321"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 44.28 | 43.78–44.91 | 22 | 473.83 |
| mode_b/component_parse/integer | 100.87 | 100.49–101.22 | 20 | 189.09 |
| mode_b/component_parse/string/borrowed | 28.29 | 28.26–28.33 | 18 | 606.73 |
| mode_b/component_parse/string/dense | 74.54 | 74.31–74.84 | 20 | 255.87 |
| mode_b/component_parse/string/late_escape | 57.21 | 56.23–58.59 | 19 | 316.73 |
| mode_b/component_parse/string/plain | 45.37 | 45.08–45.69 | 18 | 378.36 |
| mode_b/component_parse/string/unicode | 106.11 | 105.90–106.38 | 26 | 233.68 |
| mode_b/component_parse/wide/field_dispatch | 890.95 | 890.62–891.30 | 104 | 111.32 |
| mode_b/component_serialize/floats/fresh | 3682.79 | 3671.18–3696.17 | 2115 | 547.69 |
| mode_b/component_serialize/integers/fresh | 1652.06 | 1648.60–1656.73 | 1546 | 892.45 |
| mode_b/component_serialize/string/escaped | 33.22 | 32.72–33.67 | 15 | 430.67 |
| mode_b/component_serialize/string/plain | 28.21 | 27.89–28.59 | 18 | 608.43 |
| mode_b/component_serialize/wide/buffered_writer | 293.05 | 291.90–294.26 | 104 | 338.45 |
| mode_b/component_serialize/wide/direct_reuse | 274.61 | 272.15–278.89 | 104 | 361.17 |
| mode_b/component_serialize/wide/fresh | 285.65 | 285.08–286.31 | 104 | 347.22 |
| mode_b/depth/native/1 | 85.76 | 85.36–86.18 | 6 | 66.72 |
| mode_b/depth/native/127 | 9018.19 | 8934.34–9130.23 | 258 | 27.28 |
| mode_b/depth/native/16 | 750.18 | 748.29–752.03 | 36 | 45.77 |
| mode_b/depth/native/64 | 4095.40 | 4075.25–4128.11 | 132 | 30.74 |
| mode_b/depth/public_guard | 22.05 | 22.01–22.08 | — | — |
