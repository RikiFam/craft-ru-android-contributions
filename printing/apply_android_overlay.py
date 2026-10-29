"""Apply to a fresh published Android source package; no printer/device operations."""
from pathlib import Path
import argparse, hashlib, json, shutil
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('android_package', type=Path)
a=p.parse_args()
root=Path(__file__).resolve().parent
target=a.android_package.resolve()
overlay=root/'android-overlay'
m=json.loads((overlay/'manifest.json').read_bytes())
sources=json.loads((target/'sources.json').read_bytes())
assert sources['workspace_snapshot']==m['required_android_snapshot'], 'Unexpected Android package'
assert sources['component']=='android', 'Target must be the Android package'
for name, info in m['files'].items():
    data=(overlay/name).read_bytes()
    assert hashlib.sha256(data).hexdigest()==info['sha256'], name
    dest=target/name
    assert dest.resolve().is_relative_to(target), name
    actual=hashlib.sha256(dest.read_bytes()).hexdigest() if dest.exists() else None
    assert actual in (info['base_sha256'], info['sha256']), 'Modified target: '+name
for name in m['files']:
    dest=target/name
    dest.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(overlay/name,dest)
(target/'printing-overlay.json').write_text(json.dumps(m,indent=2)+'\n',encoding='utf-8')
print('Printing overlay applied. No printing, installation or cleaning was performed.')
