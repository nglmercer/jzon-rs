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
  "source_fingerprint": "4b04cfe5198565c0826414d66e7ac9c5f0d209ca8cb7a4dd0d1b1c4130deb95c"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 45.59 | 45.29–46.05 | 22 | 460.20 |
| mode_b/component_parse/integer | 101.31 | 100.82–101.96 | 20 | 188.27 |
| mode_b/component_parse/string/borrowed | 28.77 | 28.36–29.27 | 18 | 596.74 |
| mode_b/component_parse/string/dense | 83.26 | 82.73–84.00 | 20 | 229.08 |
| mode_b/component_parse/string/late_escape | 64.69 | 64.43–65.08 | 19 | 280.09 |
| mode_b/component_parse/string/plain | 41.91 | 41.72–42.15 | 18 | 409.59 |
| mode_b/component_parse/string/unicode | 120.27 | 116.59–124.61 | 26 | 206.17 |
| mode_b/component_parse/wide/field_dispatch | 743.66 | 709.02–784.48 | 104 | 133.37 |
| mode_b/component_serialize/floats/fresh | 3731.26 | 3723.91–3739.83 | 2115 | 540.57 |
| mode_b/component_serialize/integers/fresh | 1653.67 | 1631.25–1685.36 | 1546 | 891.58 |
| mode_b/component_serialize/string/escaped | 33.71 | 33.24–34.33 | 15 | 424.32 |
| mode_b/component_serialize/string/plain | 26.43 | 25.96–26.97 | 18 | 649.48 |
| mode_b/component_serialize/wide/buffered_writer | 206.44 | 203.94–209.28 | 104 | 480.43 |
| mode_b/component_serialize/wide/direct_reuse | 199.21 | 187.70–214.25 | 104 | 497.88 |
| mode_b/component_serialize/wide/fresh | 201.17 | 199.20–203.20 | 104 | 493.03 |
| mode_b/component_serialize/wide/reusable_serializer | 183.00 | 179.15–188.10 | 104 | 541.97 |
| mode_b/component_serialize/wide/streaming_to_vec | 253.37 | 251.05–256.05 | 104 | 391.45 |
| mode_b/depth/native/1 | 75.93 | 74.80–77.55 | 6 | 75.36 |
| mode_b/depth/native/127 | 8455.14 | 8327.77–8609.53 | 258 | 29.10 |
| mode_b/depth/native/16 | 724.67 | 692.88–761.40 | 36 | 47.38 |
| mode_b/depth/native/64 | 4463.17 | 4342.42–4601.84 | 132 | 28.21 |
| mode_b/depth/public_guard | 21.88 | 21.85–21.91 | — | — |
