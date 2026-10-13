# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "a01708d23ea1d748b5e084fe5b705eb6092d27d8",
  "dirty": true,
  "mode": "Mode-B-baseline-control",
  "features": "float_roundtrip",
  "cpu": "AMD BC-250",
  "os": "Linux-7.2.7-1-cachyos-x86_64-with-glibc2.44",
  "compiler": "rustc 1.97.1 (8bab26f4f 2026-07-14)\nbinary: rustc\ncommit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452\ncommit-date: 2026-07-14\nhost: x86_64-unknown-linux-gnu\nrelease: 1.97.1\nLLVM version: 22.1.6\n",
  "cargo": "cargo 1.97.1 (c980f4866 2026-06-30)",
  "rustflags": "",
  "allocator": "system default (allocation probe uses stats_alloc instrumentation)",
  "lockfile_sha256": "1dc78d69894036204819e2d847fc1b76aa764efada595c24543fb2a56a8da501",
  "datasets": {
    "crates/jzon/data/twitter.json": "a08b769f32b95f426cbc3abafcec65c1a19d3eb544d4ddf320eae142c99efc5d",
    "crates/jzon/data/mixed_2mb.json": "2da7e3b9e31d7669356f7b4216095fddaa0644c1a327dbe2c328e78e3710183c",
    "crates/jzon/data/generated_50k.json": "d4d43218e6946c0966fe532a7da0f792a7480ef7606751ffbed2a07f8f47eb56",
    "crates/jzon/data/citm_catalog.json": "a73e7a883f6ea8de113dff59702975e60119b4b58d451d518a929f31c92e2059",
    "crates/jzon/data/canada.json": "f83b3b354030d5dd58740c68ac4fecef64cb730a0d12a90362a7f23077f50d78"
  },
  "sample_settings": "30 samples, 0.2s warmup, 0.5s measurement; prior_buffered_path emulates original writer implementation",
  "source_fingerprint": "3d7d13e5e4c3edd5ee7517f0a5fa7ad6148750771d85ae8309674d43dac6fb52"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| mode_b/component_parse/float | 33.27 | 32.74–34.12 | 22 | 630.61 |
| mode_b/component_parse/integer | 72.60 | 72.42–72.83 | 20 | 262.71 |
| mode_b/component_parse/string/borrowed | 18.56 | 18.50–18.65 | 18 | 925.03 |
| mode_b/component_parse/string/dense | 58.46 | 58.25–58.69 | 20 | 326.24 |
| mode_b/component_parse/string/late_escape | 62.43 | 62.33–62.54 | 19 | 290.25 |
| mode_b/component_parse/string/plain | 29.86 | 29.62–30.02 | 18 | 574.97 |
| mode_b/component_parse/string/unicode | 124.61 | 124.17–125.09 | 26 | 198.99 |
| mode_b/component_parse/wide/field_dispatch | 306.51 | 303.41–310.81 | 104 | 323.59 |
| mode_b/component_serialize/floats/fresh | 6840.09 | 6828.38–6859.13 | 2115 | 294.88 |
| mode_b/component_serialize/integers/fresh | 1638.53 | 1637.21–1639.92 | 1546 | 899.82 |
| mode_b/component_serialize/string/escaped | 30.00 | 29.26–30.82 | 15 | 476.77 |
| mode_b/component_serialize/string/plain | 25.59 | 25.54–25.64 | 18 | 670.79 |
| mode_b/component_serialize/wide/buffered_writer | 285.66 | 285.36–285.97 | 104 | 347.20 |
| mode_b/component_serialize/wide/fresh | 281.50 | 280.51–282.48 | 104 | 352.34 |
| mode_b/component_serialize/wide/prior_buffered_path | 285.38 | 285.00–285.74 | 104 | 347.55 |
