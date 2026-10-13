# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "94727f756af4db62c6ce5892ea4b650d098622be",
  "dirty": true,
  "mode": "A/B optimization serialize-after-1",
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
  "sample_settings": "200 samples/1s warmup/4s measurement",
  "source_fingerprint": "45b17eeff552ab63e5dfef7fc0535fe9e90d8a9389f1b84c84c1d84993619e53"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| serialize/canada/jzon | 4528701.70 | 4500143.14–4559894.88 | 2090326 | 440.19 |
| serialize/canada/jzon_serde | 3775165.03 | 3720784.93–3835368.21 | 2090326 | 528.05 |
| serialize/canada/serde_json | 3892024.87 | 3858520.45–3925795.95 | 2090326 | 512.20 |
| serialize/canada/sonic_rs | 4689366.35 | 4669601.60–4712253.76 | 2090326 | 425.11 |
| serialize/twitter/jzon | 37186.23 | 36646.45–37772.15 | 51013 | 1308.27 |
| serialize/twitter/jzon_serde | 99056.77 | 97823.50–100394.21 | 51013 | 491.13 |
| serialize/twitter/serde_json | 50284.17 | 49920.74–50703.59 | 51013 | 967.50 |
| serialize/twitter/sonic_rs | 24009.60 | 23872.82–24161.27 | 51013 | 2026.26 |
