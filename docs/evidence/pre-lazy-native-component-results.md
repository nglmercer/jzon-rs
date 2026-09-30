# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "a01708d23ea1d748b5e084fe5b705eb6092d27d8",
  "dirty": true,
  "mode": "Mode-B-component-profile",
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
  "sample_settings": "30 samples, 0.2s warmup, 0.5s measurement per case",
  "source_fingerprint": "6e5b25a37b2041939181d0b971eebe59512cf7ce456048924d3b685bab755fa5"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 52.67 | 51.81–53.99 | 22 | 398.31 |
| mode_b/component_parse/integer | 112.81 | 111.43–114.70 | 20 | 169.07 |
| mode_b/component_parse/string/borrowed | 37.45 | 37.23–37.74 | 18 | 458.38 |
| mode_b/component_parse/string/dense | 92.14 | 91.97–92.32 | 20 | 207.00 |
| mode_b/component_parse/string/late_escape | 63.18 | 62.75–63.64 | 19 | 286.79 |
| mode_b/component_parse/string/plain | 50.33 | 50.24–50.42 | 18 | 341.06 |
| mode_b/component_parse/string/unicode | 107.18 | 106.82–107.53 | 26 | 231.35 |
| mode_b/component_parse/wide/field_dispatch | 781.90 | 776.09–789.66 | 104 | 126.85 |
| mode_b/component_serialize/floats/fresh | 3706.89 | 3663.30–3767.97 | 2115 | 544.13 |
| mode_b/component_serialize/integers/fresh | 1698.37 | 1675.18–1724.66 | 1546 | 868.11 |
| mode_b/component_serialize/string/escaped | 35.35 | 33.59–37.26 | 15 | 404.62 |
| mode_b/component_serialize/string/plain | 30.14 | 29.63–30.76 | 18 | 569.56 |
| mode_b/component_serialize/wide/buffered_writer | 309.10 | 298.63–323.17 | 104 | 320.87 |
| mode_b/component_serialize/wide/direct_reuse | 317.68 | 300.91–335.93 | 104 | 312.21 |
| mode_b/component_serialize/wide/fresh | 289.63 | 286.72–294.60 | 104 | 342.45 |
