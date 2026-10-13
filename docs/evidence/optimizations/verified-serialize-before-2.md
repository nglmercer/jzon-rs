# Current benchmark invocation

Historical results are never merged. Modes and configurations remain separate.

```json
{
  "commit": "94727f756af4db62c6ce5892ea4b650d098622be",
  "dirty": true,
  "mode": "A/B optimization serialize-before-2",
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
| serialize/canada/jzon | 4433392.62 | 4417116.01–4453875.99 | 2090326 | 449.65 |
| serialize/canada/jzon_serde | 3862800.51 | 3841576.50–3887622.41 | 2090326 | 516.07 |
| serialize/canada/serde_json | 3414802.58 | 3394913.93–3436342.92 | 2090326 | 583.78 |
| serialize/canada/sonic_rs | 4415223.44 | 4388718.56–4445349.79 | 2090326 | 451.50 |
| serialize/twitter/jzon | 51382.40 | 51126.54–51662.08 | 51013 | 946.82 |
| serialize/twitter/jzon_serde | 93824.08 | 93216.88–94499.02 | 51013 | 518.52 |
| serialize/twitter/serde_json | 51719.99 | 50893.35–52579.76 | 51013 | 940.64 |
| serialize/twitter/sonic_rs | 26213.41 | 25848.11–26606.49 | 51013 | 1855.91 |
