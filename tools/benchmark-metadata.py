#!/usr/bin/env python3
import hashlib,json,os,pathlib,platform,subprocess,sys
cpu=platform.processor()
if pathlib.Path('/proc/cpuinfo').exists():
    cpu=next((line.split(':',1)[1].strip() for line in pathlib.Path('/proc/cpuinfo').read_text().splitlines() if line.startswith('model name')),cpu)
metadata=dict(commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),dirty=bool(subprocess.check_output(['git','status','--porcelain'],text=True)),mode=sys.argv[1],features=sys.argv[2],cpu=cpu,os=platform.platform(),compiler=subprocess.check_output(['rustc','-Vv'],text=True),cargo=subprocess.check_output(['cargo','-V'],text=True).strip(),rustflags=os.environ.get('RUSTFLAGS',''),allocator='system default (allocation probe uses stats_alloc instrumentation)',lockfile_sha256=hashlib.sha256(pathlib.Path('Cargo.lock').read_bytes()).hexdigest(),datasets={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in pathlib.Path('crates/jzon/data').glob('*.json')},sample_settings=sys.argv[3],source_fingerprint=hashlib.sha256(b''.join(str(p).encode()+p.read_bytes() for p in sorted(pathlib.Path('crates').rglob('*.rs')) if 'target' not in p.parts)).hexdigest())
print(json.dumps(metadata,indent=2))
