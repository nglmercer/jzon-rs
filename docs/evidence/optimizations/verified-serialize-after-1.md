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
  "source_fingerprint": "8e17cb53d13682e0eb12a6544c020ba9c64bc96be17b1d66adb18f1e6cadf6aa"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| serialize/canada/jzon | 4393568.39 | 4384707.90–4405441.55 | 2090326 | 453.73 |
| serialize/canada/jzon_serde | 3544635.98 | 3536607.42–3553510.68 | 2090326 | 562.40 |
| serialize/canada/serde_json | 3424103.08 | 3416710.22–3431313.46 | 2090326 | 582.19 |
| serialize/canada/sonic_rs | 4288521.53 | 4279835.41–4300302.97 | 2090326 | 464.84 |
| serialize/twitter/jzon | 42130.94 | 42071.41–42193.64 | 51013 | 1154.73 |
| serialize/twitter/jzon_serde | 83001.73 | 82944.88–83064.89 | 51013 | 586.13 |
| serialize/twitter/serde_json | 48389.99 | 48293.81–48520.78 | 51013 | 1005.37 |
| serialize/twitter/sonic_rs | 23207.46 | 23164.21–23257.05 | 51013 | 2096.30 |
