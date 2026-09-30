#!/usr/bin/env python3
from pathlib import Path
import shutil
for path in Path('crates/jzon/tests/corpus').glob('[yni]_*.json'):
    for target in ['native','map_keys','skip','raw','roundtrip']:
        destination=Path('fuzz/corpus')/target/('suite-'+path.name)
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(path,destination)

# Typed key seeds exercise paths that Value maps cannot reach.
key_seeds = [
    b'{:1}', b'{ \t :1}', b'{"Unit":1,{"New":1}:2}',
    b'{"Unit":1,:2}', b'{"Unit":1,}', b'{"Unit":1,',
    b'{"Unit":1}', b'{"true":1,"false":2}',
    br'{"\u0074rue":1}', br'{"t\u0072ue":1}', b'{"truex":1}',
    b'{"true:1}', '{"é":1}'.encode(), b'{"abc":1}', br'{"a\u0062c":1}',
    br'{"\uD800":1}', br'{"\uDC00":1}', br'{"\uD83D\uDE00":1}',
    b'{"\xff":1}', b'{"\t":1}', br'{"\q":1}', b'{"abc":1,[1]:2}',
]
for index, data in enumerate(key_seeds):
    destination = Path('fuzz/corpus/map_keys') / f'review-key-{index:02}'
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(data)
