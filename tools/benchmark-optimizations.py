#!/usr/bin/env python3
"""Compare isolated committed baseline and worktree, keeping every invocation."""
import argparse,json,os,pathlib,subprocess,time
parser=argparse.ArgumentParser()
parser.add_argument('--baseline',type=pathlib.Path,required=True)
parser.add_argument('--out',type=pathlib.Path,required=True)
parser.add_argument('--repeats',type=int,default=2)
a=parser.parse_args();a.out=a.out.resolve();a.out.mkdir(parents=True,exist_ok=True)
repo=pathlib.Path(__file__).resolve().parent.parent
baseline=a.baseline.resolve()
results=[]
for repetition in range(a.repeats):
    for suite,target,arguments,settings in [
        ('components','bench_native_profile',['--sample-size','30','--warm-up-time','0.2','--measurement-time','0.5'],'30 samples/.2s warmup/.5s measurement'),
        ('serialize','bench_cmp',['^serialize/(twitter|canada)'],'200 samples/1s warmup/4s measurement'),
    ]:
        for state,cwd in [('before',baseline),('after',repo)]:
            label=f'{suite}-{state}-{repetition+1}'
            root=a.out/label;metadata=a.out/(label+'-metadata.json');log=a.out/(label+'.log')
            with metadata.open('w') as stream:
                subprocess.run(['python3',str(repo/'tools/benchmark-metadata.py'),f'A/B optimization {label}','float_roundtrip',settings],cwd=cwd,stdout=stream,check=True)
            command=['cargo','bench','-p','jzon-rs','--bench',target,'--features','float_roundtrip','--',*arguments]
            start=time.monotonic()
            with log.open('w') as stream:
                result=subprocess.run(command,cwd=cwd,env=dict(os.environ,CRITERION_HOME=str(root)),stdout=stream,stderr=subprocess.STDOUT)
            entry=dict(label=label,command=command,cwd=str(cwd),exit_status=result.returncode,elapsed_seconds=time.monotonic()-start,log=str(log))
            results.append(entry);(a.out/'commands.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(entry),flush=True)
            if result.returncode:raise SystemExit(result.returncode)
            raw={str(p.relative_to(root)):json.loads(p.read_text()) for p in root.glob('**/new/*.json')}
            if not raw:raise SystemExit(f'No estimates for {label}')
            (a.out/(label+'-estimates.json')).write_text(json.dumps(raw,indent=2)+'\n')
            subprocess.run(['python3',str(repo/'tools/benchmark-report.py'),str(root),'--metadata',str(metadata),'--output',str(a.out/(label+'.md'))],check=True)
