# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "94727f756af4db62c6ce5892ea4b650d098622be",
  "dirty": true,
  "mode": "A/B optimization serialize-before-1",
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
  "sample_settings": "200 samples/1s warmup/4s measurement",
  "source_fingerprint": "ef7d2ce6ca6543f0dc52c68122920d09464c922d972d4dd946c0ea9509e31321"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| serialize/canada/jzon | 4362738.31 | 4358888.19–4367435.46 | 2090326 | 456.94 |
| serialize/canada/jzon_serde | 3768414.15 | 3754822.63–3783769.30 | 2090326 | 529.00 |
| serialize/canada/serde_json | 3342085.64 | 3337274.99–3347346.81 | 2090326 | 596.48 |
| serialize/canada/sonic_rs | 4293035.19 | 4277841.64–4318942.74 | 2090326 | 464.35 |
| serialize/twitter/jzon | 50175.65 | 50057.79–50326.21 | 51013 | 969.59 |
| serialize/twitter/jzon_serde | 90495.09 | 90372.36–90696.11 | 51013 | 537.60 |
| serialize/twitter/serde_json | 46365.21 | 46201.14–46557.96 | 51013 | 1049.27 |
| serialize/twitter/sonic_rs | 23111.56 | 23018.97–23233.01 | 51013 | 2105.00 |
