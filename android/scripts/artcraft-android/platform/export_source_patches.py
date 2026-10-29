"""Capture exact Android source edits relative to the installed Russian update."""
from pathlib import Path
import json
import subprocess

root = Path(__file__).resolve().parents[3]
base = Path(__file__).resolve().parent
versions = json.loads((root/'scripts/artcraft-russian/update-2026-10-29.json').read_text('utf-8-sig'))
output = base/'source-patches'
output.mkdir(exist_ok=True)
manifest = {}
for app, version in versions.items():
    if app == 'craft-launcher':
        continue
    repo = root/f'tmp/artcraft/{app}/repo'
    if not repo.is_dir():
        continue
    revision = version['local_commit']
    diff = subprocess.check_output(['git', '-C', str(repo), 'diff', '--binary', revision, '--', 'crates'])
    if diff:
        (output/f'{app}.patch').write_bytes(diff)
        manifest[app] = {'base': revision, 'patch': f'{app}.patch'}
(output/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n', 'utf-8')
