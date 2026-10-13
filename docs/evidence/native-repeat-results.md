# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "a01708d23ea1d748b5e084fe5b705eb6092d27d8",
  "dirty": true,
  "mode": "A/B projected Twitter and Canada serialization",
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
| serialize/canada/jzon | 4363572.77 | 4360339.32–4368856.62 | 2090326 | 456.85 |
| serialize/canada/jzon_serde | 3733899.41 | 3732194.54–3735629.46 | 2090326 | 533.89 |
| serialize/canada/serde_json | 3335218.06 | 3327010.19–3344501.33 | 2090326 | 597.71 |
| serialize/canada/sonic_rs | 4334666.93 | 4321169.23–4350981.74 | 2090326 | 459.89 |
| serialize/twitter/jzon | 49514.24 | 49501.17–49529.11 | 51013 | 982.54 |
| serialize/twitter/jzon_serde | 90728.80 | 90618.24–90854.06 | 51013 | 536.21 |
| serialize/twitter/serde_json | 48243.46 | 47787.94–48726.16 | 51013 | 1008.42 |
| serialize/twitter/sonic_rs | 23173.35 | 23082.53–23288.99 | 51013 | 2099.39 |
