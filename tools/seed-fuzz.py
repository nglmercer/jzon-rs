#!/usr/bin/env python3
from pathlib import Path
import shutil
for path in Path('crates/jzon/tests/corpus').glob('[yni]_*.json'):
    for target in ['native','skip','raw','roundtrip']:
        destination=Path('fuzz/corpus')/target/('suite-'+path.name)
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(path,destination)
