"""Apply only after the documented printing and spelling overlays; no device access."""
from pathlib import Path
import hashlib,json,shutil
root=Path(__file__).resolve().parent;overlay=root/'dex-overlay'
m=json.loads((overlay/'manifest.json').read_bytes())
assert json.loads((root/'printing-overlay.json').read_bytes())['snapshot']==m['required_printing_snapshot'], 'Apply printing overlay first'
assert (root/'scripts/artcraft-spelling/NOTICE').is_file(), 'Apply spelling overlay first'
for name,entry in m['files'].items():
 data=(overlay/name).read_bytes();assert hashlib.sha256(data).hexdigest()==entry['sha256'],name
 dest=root/name;actual=hashlib.sha256(dest.read_bytes()).hexdigest() if dest.exists() else None
 assert actual in (entry['base_sha256'],entry['sha256']), 'Unexpected or modified input: '+name
for name in m['files']:
 dest=root/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(overlay/name,dest)
print('DeX/input overlay applied; rebuild all selected native hosts. No device contacted.')
