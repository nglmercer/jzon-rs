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
  "source_fingerprint": "4b04cfe5198565c0826414d66e7ac9c5f0d209ca8cb7a4dd0d1b1c4130deb95c"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 45.57 | 45.33–45.79 | 22 | 460.42 |
| mode_b/component_parse/integer | 101.97 | 101.34–102.68 | 20 | 187.05 |
| mode_b/component_parse/string/borrowed | 22.53 | 21.99–23.20 | 18 | 761.86 |
| mode_b/component_parse/string/dense | 94.08 | 91.04–97.22 | 20 | 202.73 |
| mode_b/component_parse/string/late_escape | 69.61 | 67.61–72.04 | 19 | 260.32 |
| mode_b/component_parse/string/plain | 43.47 | 42.95–44.22 | 18 | 394.85 |
| mode_b/component_parse/string/unicode | 119.95 | 116.09–124.62 | 26 | 206.71 |
| mode_b/component_parse/wide/field_dispatch | 731.30 | 704.35–761.08 | 104 | 135.62 |
| mode_b/component_serialize/floats/fresh | 3879.22 | 3792.48–3979.71 | 2115 | 519.96 |
| mode_b/component_serialize/integers/fresh | 1764.57 | 1715.63–1820.32 | 1546 | 835.54 |
| mode_b/component_serialize/string/escaped | 36.19 | 35.06–37.66 | 15 | 395.31 |
| mode_b/component_serialize/string/plain | 26.30 | 25.72–27.00 | 18 | 652.58 |
| mode_b/component_serialize/wide/buffered_writer | 199.45 | 197.88–200.89 | 104 | 497.27 |
| mode_b/component_serialize/wide/direct_reuse | 187.57 | 183.77–191.99 | 104 | 528.77 |
| mode_b/component_serialize/wide/fresh | 196.37 | 193.70–199.63 | 104 | 505.07 |
| mode_b/component_serialize/wide/reusable_serializer | 185.38 | 181.65–190.22 | 104 | 535.01 |
| mode_b/component_serialize/wide/streaming_to_vec | 256.03 | 250.58–262.16 | 104 | 387.38 |
| mode_b/depth/native/1 | 73.93 | 73.77–74.10 | 6 | 77.40 |
| mode_b/depth/native/127 | 8619.48 | 8545.05–8719.31 | 258 | 28.55 |
| mode_b/depth/native/16 | 658.41 | 649.48–671.77 | 36 | 52.14 |
| mode_b/depth/native/64 | 4205.87 | 4177.49–4237.63 | 132 | 29.93 |
| mode_b/depth/public_guard | 22.78 | 22.76–22.80 | — | — |
