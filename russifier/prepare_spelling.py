"""Prepare spelling on the sources fetched by prepare_sources.py; no installation."""
from pathlib import Path
import subprocess, sys
root=Path(__file__).resolve().parent
for name in ('prepare.py','prepare_panels.py','prepare_tests.py','finish_sources.py'):
    subprocess.run([sys.executable,str(root/'scripts/artcraft-spelling'/name)],cwd=root,check=True)
print('Spelling sources and notices prepared; no binaries installed.')
