#!/usr/bin/env python3
"""Measure one isolated executable (Linux ru_maxrss is KiB)."""
import json,resource,subprocess,sys,time
start=time.monotonic()
result=subprocess.run([sys.argv[1]],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
usage=resource.getrusage(resource.RUSAGE_CHILDREN)
print(result.stdout,end='')
print(json.dumps(dict(exit_status=result.returncode,elapsed_seconds=time.monotonic()-start,peak_rss_kib=usage.ru_maxrss,user_seconds=usage.ru_utime,system_seconds=usage.ru_stime)),file=sys.stderr)
if result.stderr:print(result.stderr,file=sys.stderr)
sys.exit(result.returncode)
