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
| mode_b/component_parse/float | 68.37 | 64.98–72.10 | 22 | 306.88 |
| mode_b/component_parse/integer | 142.97 | 140.67–145.74 | 20 | 133.41 |
| mode_b/component_parse/string/borrowed | 44.12 | 42.62–45.82 | 18 | 389.04 |
| mode_b/component_parse/string/dense | 125.04 | 115.17–136.60 | 20 | 152.53 |
| mode_b/component_parse/string/late_escape | 134.45 | 120.81–148.41 | 19 | 134.77 |
| mode_b/component_parse/string/plain | 74.20 | 69.96–79.61 | 18 | 231.34 |
| mode_b/component_parse/string/unicode | 185.29 | 165.50–207.10 | 26 | 133.82 |
| mode_b/component_parse/wide/field_dispatch | 1012.52 | 973.86–1053.16 | 104 | 97.96 |
| mode_b/component_serialize/floats/fresh | 4030.14 | 3909.97–4169.47 | 2115 | 500.48 |
| mode_b/component_serialize/integers/fresh | 3518.61 | 2914.91–4254.68 | 1546 | 419.02 |
| mode_b/component_serialize/string/escaped | 42.42 | 40.33–44.50 | 15 | 337.20 |
| mode_b/component_serialize/string/plain | 34.96 | 33.62–36.29 | 18 | 491.06 |
| mode_b/component_serialize/wide/buffered_writer | 325.26 | 311.48–340.96 | 104 | 304.93 |
| mode_b/component_serialize/wide/direct_reuse | 286.68 | 280.78–292.75 | 104 | 345.97 |
| mode_b/component_serialize/wide/fresh | 375.36 | 359.84–390.89 | 104 | 264.23 |
