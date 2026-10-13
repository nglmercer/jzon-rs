#!/usr/bin/env python3
"""Render only estimates from this invocation's Criterion directory."""
import argparse, json, pathlib
p=argparse.ArgumentParser()
p.add_argument('root',type=pathlib.Path)
p.add_argument('--metadata',type=pathlib.Path,required=True)
p.add_argument('--output',type=pathlib.Path,required=True)
a=p.parse_args()
lines=['# Current benchmark invocation','', 'Historical results are never merged. Modes and configurations remain separate.','', '```json',a.metadata.read_text(),'```','', '| Benchmark | Time/op mean (ns) | 95% CI (ns) | Output/input bytes | MiB/s |', '|---|---:|---:|---:|---:|']
for file in sorted(a.root.glob('**/new/estimates.json')):
    estimate=json.loads(file.read_text())['mean']
    config=json.loads(file.with_name('benchmark.json').read_text())
    throughput=config.get('throughput') or {}
    size=throughput.get('Bytes')
    mean=estimate['point_estimate'];ci=estimate['confidence_interval']
    mib=f'{size/(mean/1e9)/2**20:.2f}' if size else '—'
    lines.append(f"| {config['full_id']} | {mean:.2f} | {ci['lower_bound']:.2f}–{ci['upper_bound']:.2f} | {size or '—'} | {mib} |")
a.output.write_text('\n'.join(lines)+'\n')
