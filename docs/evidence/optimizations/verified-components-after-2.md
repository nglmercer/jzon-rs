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
  "source_fingerprint": "8e17cb53d13682e0eb12a6544c020ba9c64bc96be17b1d66adb18f1e6cadf6aa"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 47.15 | 45.47–49.16 | 22 | 445.02 |
| mode_b/component_parse/integer | 102.14 | 101.19–103.47 | 20 | 186.73 |
| mode_b/component_parse/string/borrowed | 25.18 | 24.48–25.94 | 18 | 681.86 |
| mode_b/component_parse/string/dense | 84.12 | 83.72–84.55 | 20 | 226.75 |
| mode_b/component_parse/string/late_escape | 64.60 | 64.26–65.10 | 19 | 280.51 |
| mode_b/component_parse/string/plain | 42.27 | 41.90–42.78 | 18 | 406.09 |
| mode_b/component_parse/string/unicode | 109.45 | 109.35–109.56 | 26 | 226.54 |
| mode_b/component_parse/wide/field_dispatch | 867.64 | 835.01–899.26 | 104 | 114.31 |
| mode_b/component_serialize/floats/fresh | 3959.88 | 3861.57–4069.36 | 2115 | 509.36 |
| mode_b/component_serialize/integers/fresh | 1785.00 | 1746.91–1828.84 | 1546 | 825.98 |
| mode_b/component_serialize/string/escaped | 35.63 | 34.90–36.40 | 15 | 401.54 |
| mode_b/component_serialize/string/plain | 27.59 | 26.44–29.03 | 18 | 622.11 |
| mode_b/component_serialize/wide/buffered_writer | 184.07 | 178.50–190.29 | 104 | 538.82 |
| mode_b/component_serialize/wide/direct_reuse | 185.01 | 176.04–194.62 | 104 | 536.09 |
| mode_b/component_serialize/wide/fresh | 181.45 | 176.38–187.17 | 104 | 546.62 |
| mode_b/component_serialize/wide/reusable_serializer | 171.44 | 164.25–179.29 | 104 | 578.53 |
| mode_b/component_serialize/wide/streaming_to_vec | 236.35 | 234.27–238.69 | 104 | 419.64 |
| mode_b/depth/native/1 | 79.10 | 76.99–81.52 | 6 | 72.34 |
| mode_b/depth/native/127 | 8735.69 | 8585.99–8905.72 | 258 | 28.17 |
| mode_b/depth/native/16 | 711.28 | 686.16–740.85 | 36 | 48.27 |
| mode_b/depth/native/64 | 4092.26 | 4016.65–4189.36 | 132 | 30.76 |
| mode_b/depth/public_guard | 21.93 | 21.89–21.97 | — | — |
