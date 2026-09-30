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
  "source_fingerprint": "b609a81d383da50e2c3f11670fbac750f0f40eb508956b5ee96604c327de59b3"
}

```

| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |
|---|---:|---:|---:|---:|
| serialize/canada/jzon | 4512222.98 | 4483294.47–4544126.55 | 2090326 | 441.80 |
| serialize/canada/jzon_serde | 3912741.80 | 3883999.15–3944860.34 | 2090326 | 509.49 |
| serialize/canada/serde_json | 3354946.95 | 3345769.15–3365046.99 | 2090326 | 594.19 |
| serialize/canada/sonic_rs | 4376672.45 | 4357041.30–4399094.96 | 2090326 | 455.48 |
| serialize/twitter/jzon | 52124.52 | 51707.01–52576.21 | 51013 | 933.34 |
| serialize/twitter/jzon_serde | 95502.26 | 94868.67–96196.95 | 51013 | 509.41 |
| serialize/twitter/serde_json | 46697.70 | 46552.85–46858.50 | 51013 | 1041.80 |
| serialize/twitter/sonic_rs | 23692.81 | 23525.98–23894.16 | 51013 | 2053.36 |
