# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "a01708d23ea1d748b5e084fe5b705eb6092d27d8",
  "dirty": true,
  "mode": "B native component profile",
  "features": "float_roundtrip",
  "cpu": "AMD BC-250",
  "os": "Linux-7.2.7-1-cachyos-x86_64-with-glibc2.44",
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
  "sample_settings": "30 samples/.2s/.5s profile; 200/1s/4s native; 30/.5s/1s compat",
  "source_fingerprint": "2334a34b4a23bc79e8a92243bef6528ec64f62be1020976b2bf6db80decfa96c"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 43.43 | 42.97–44.00 | 22 | 483.09 |
| mode_b/component_parse/integer | 103.56 | 102.36–104.97 | 20 | 184.18 |
| mode_b/component_parse/string/borrowed | 28.90 | 28.07–29.98 | 18 | 593.99 |
| mode_b/component_parse/string/dense | 76.44 | 74.54–79.34 | 20 | 249.51 |
| mode_b/component_parse/string/late_escape | 57.47 | 56.20–59.06 | 19 | 315.28 |
| mode_b/component_parse/string/plain | 41.79 | 41.19–42.53 | 18 | 410.77 |
| mode_b/component_parse/string/unicode | 100.85 | 99.64–102.08 | 26 | 245.87 |
| mode_b/component_parse/wide/field_dispatch | 765.93 | 751.76–782.09 | 104 | 129.49 |
| mode_b/component_serialize/floats/fresh | 3738.34 | 3719.52–3758.47 | 2115 | 539.55 |
| mode_b/component_serialize/integers/fresh | 1704.70 | 1689.28–1722.61 | 1546 | 864.89 |
| mode_b/component_serialize/string/escaped | 34.47 | 33.43–35.80 | 15 | 414.95 |
| mode_b/component_serialize/string/plain | 29.95 | 29.44–30.58 | 18 | 573.24 |
| mode_b/component_serialize/wide/buffered_writer | 289.72 | 286.22–294.20 | 104 | 342.34 |
| mode_b/component_serialize/wide/direct_reuse | 275.27 | 271.69–279.62 | 104 | 360.31 |
| mode_b/component_serialize/wide/fresh | 333.51 | 315.33–353.61 | 104 | 297.39 |
