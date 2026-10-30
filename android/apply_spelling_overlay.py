"""Apply Android spelling files and copy reviewed spelling sources from russifier."""
from pathlib import Path
import argparse,hashlib,json,shutil
p=argparse.ArgumentParser(description=__doc__);p.add_argument('russifier',type=Path);a=p.parse_args()
root=Path(__file__).resolve().parent;source=a.russifier.resolve();overlay=root/'spelling-overlay'
m=json.loads((overlay/'manifest.json').read_bytes())
assert json.loads((root/'printing-overlay.json').read_bytes())['snapshot']==m['requires_printing_overlay'], 'Apply printing overlay first'
assert json.loads((source/'sources.json').read_bytes())['spelling_snapshot']==m['snapshot'], 'Wrong spelling source package'
for name,info in m['files'].items():
    assert hashlib.sha256((overlay/name).read_bytes()).hexdigest()==info['sha256'],name
    assert hashlib.sha256((root/name).read_bytes()).hexdigest() in (info['base_sha256'],info['sha256']), 'Modified target: '+name
copy=[]
for folder in ('scripts/artcraft-spelling','scripts/artcraft-russian/dictionaries'):
    copy.extend(p for p in (source/folder).rglob('*') if p.is_file() and '__pycache__' not in p.parts)
copy.append(source/'prepare_spelling.py')
for path in copy:
    dest=root/path.relative_to(source)
    assert not dest.exists() or dest.read_bytes()==path.read_bytes(), 'Modified target: '+str(dest)
for name in m['files']:
    shutil.copyfile(overlay/name,root/name)
for path in copy:
    dest=root/path.relative_to(source);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(path,dest)
print('Android spelling source overlay applied. Run prepare_spelling.py after preparing source trees.')
