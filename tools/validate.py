#!/usr/bin/env python3
"""Execute validation and preserve every process's actual status and log."""
import argparse,json,pathlib,subprocess,time,shlex
p=argparse.ArgumentParser();p.add_argument('suite',choices=['core','safety','msrv','release','fuzz']);p.add_argument('--out',type=pathlib.Path,default=pathlib.Path('/tmp/jzon-readiness-evidence/final'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
suites={
'core':[
'cargo fmt --all -- --check',
'cargo test --workspace',
'cargo clippy --workspace --all-targets -- -D warnings',
'cargo test -p jzon-rs --features compat',
'cargo test -p jzon-rs --features compat,raw_value,preserve_order,arbitrary_precision,float_roundtrip',
'cargo test -p jzon-rs --features compat,unbounded_depth',
'cargo test -p jzon-rs --features compat,simd,simd-intrinsics,stats,zmij-float-ser',
'cargo test -p jzon-rs-compat',
'cargo test -p jzon-rs-serde',
'cargo doc --workspace --no-deps',
'python3 tools/check-consumers.py',
'cargo check --manifest-path fixtures/alloc/Cargo.toml --locked --target thumbv7em-none-eabi',
'cargo check -p jzon-rs-serde --no-default-features',
'cargo check -p jzon-rs-compat --no-default-features --features alloc',
'cargo bench -p jzon-rs --bench bench_cmp --features float_roundtrip -- --test',
],
'safety':[
'cargo +nightly miri test -p jzon-rs --test depth_safety --test readiness',
'cargo +nightly test --workspace --features simd,unstable',
],
'msrv':[
f'cargo +1.71.0 check --manifest-path fixtures/{name}/Cargo.toml --locked' for name in ['facade','native','alloc','derive']
],
'release':['cargo audit --json','cargo package --workspace --allow-dirty'],
'fuzz':[]}
results=[]
commands=[(cmd,None) for cmd in suites[a.suite]]
if a.suite=='fuzz':
    for policy in ['', '--features roundtrip_policy']:
        for target in ['native','skip','raw','roundtrip','simd']:
            commands.append((f'cargo +nightly fuzz run {target} {policy} -- -max_total_time=5 -max_len=4096 -rss_limit_mb=512','fuzz'))
for index,(command,cwd) in enumerate(commands):
    start=time.monotonic();log=a.out/f'{a.suite}-{index:02}.log'
    with log.open('w') as output:
        result=subprocess.run(shlex.split(command),cwd=cwd,stdout=output,stderr=subprocess.STDOUT)
    entry=dict(command=command,cwd=cwd or '.',exit_status=result.returncode,elapsed_seconds=round(time.monotonic()-start,3),log=str(log))
    results.append(entry);(a.out/f'{a.suite}-results.json').write_text(json.dumps(results,indent=2)+'\n')
    print(json.dumps(entry),flush=True)
raise SystemExit(int(any(result['exit_status'] for result in results)))
