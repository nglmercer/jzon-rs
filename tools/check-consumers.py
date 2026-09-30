#!/usr/bin/env python3
import subprocess,sys
commands=[['cargo','run','--manifest-path','fixtures/facade/Cargo.toml','--locked'],['cargo','run','--manifest-path','fixtures/facade/Cargo.toml','--locked','--features','mirrors'],['cargo','run','--manifest-path','fixtures/native/Cargo.toml','--locked'],['cargo','check','--manifest-path','fixtures/alloc/Cargo.toml','--locked']]
commands.append(['cargo','run','--manifest-path','fixtures/interop/Cargo.toml','--locked'])
commands.append(['cargo','run','--manifest-path','fixtures/derive/Cargo.toml','--locked'])
for cmd in commands:
    subprocess.run(cmd,check=True)
for name,message in [('borrow_outlives','does not live long enough'),('unsupported_attr','unsupported Mode A container attribute')]:
    result=subprocess.run(['cargo','check','--manifest-path','fixtures/compile_fail/Cargo.toml','--bin',name],text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    if result.returncode==0 or message not in result.stdout:
        print(result.stdout);sys.exit(f'expected compile rejection missing: {name}')
    print(f'PASS expected compile rejection: {name}')
