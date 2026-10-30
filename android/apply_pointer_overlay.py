"""Apply after printing, spelling and DeX overlays; validate before writing."""
from pathlib import Path
import hashlib,json,shutil
root=Path(__file__).resolve().parent
overlay=root/'pointer-overlay'
m=json.loads((overlay/'manifest.json').read_bytes())
for name,e in m['files'].items():
 data=(overlay/name).read_bytes()
 assert hashlib.sha256(data).hexdigest()==e['sha256'],name
 dest=root/name
 assert dest.is_file(), 'Apply earlier overlays first: '+name
 assert hashlib.sha256(dest.read_bytes()).hexdigest() in (e['base_sha256'],e['sha256']), 'Unexpected input: '+name
for name in m['files']:
 shutil.copyfile(overlay/name,root/name)
print('Pointer origin overlay applied; perform a full native rebuild. No device contacted.')
