"""Match Android dependency forks and package versions to the new desktop release."""
import json
import os
import re
import tomllib
from pathlib import Path

BASE = Path(__file__).resolve().parent
ROOT = BASE.parents[1]
state = json.loads((ROOT/'scripts/artcraft-russian/update-2026-10-29.json').read_text('utf-8'))
for app, item in state.items():
    host = BASE if app == 'photocraft' else BASE/'suite'/('launcher' if app == 'craft-launcher' else app)
    repo = ROOT/'tmp/artcraft'/app/'repo'
    cargo = host/'Cargo.toml'
    text = cargo.read_text('utf-8').split('[patch.crates-io]')[0].rstrip()
    text = text.replace('version = "=0.36.2"', 'version = "0.37"')
    patches = tomllib.loads((repo/'Cargo.toml').read_text('utf-8'))['patch']['crates-io']
    if app == 'pdfcraft':
        patches['rfd'] = {'path': str(ROOT/'tmp/artcraft/android/rfd-android')}
    text += '\n\n[patch.crates-io]\n'
    for name, spec in patches.items():
        if 'path' in spec:
            spec['path'] = os.path.relpath(repo/spec['path'],host).replace('\\','/')
        text += name+' = { '+', '.join(k+' = '+json.dumps(v) for k,v in spec.items())+' }\n'
    cargo.write_text(text,'utf-8')
    manifest = host/'AndroidManifest.xml'
    text = manifest.read_text('utf-8')
    text = re.sub(r'android:versionCode="\d+"','android:versionCode="3"',text)
    text = re.sub(r'android:versionName="[^"]+"','android:versionName="'+item['tag'].lstrip('v')+'-ru-android-preview3"',text)
    manifest.write_text(text,'utf-8')
    ui = repo/'crates/ui-egui/Cargo.toml'
    text = ui.read_text('utf-8')
    lines = text.splitlines()
    for i,line in enumerate(lines):
        if line.startswith('eframe =') and '"clipboard"' in line:
            lines[i] = line.replace(', "accesskit"','').replace(', "clipboard"','')
            # Desktop integration remains explicitly enabled off Android.
            text='\n'.join(lines)+'\n'
            section='[target.\'cfg(not(target_os = "android"))\'.dependencies]'
            if section not in text:
                text+='\n'+section+'\neframe = { workspace = true, features = ["accesskit", "clipboard"] }\n'
            else:
                text=text.replace('features = ["accesskit"]','features = ["accesskit", "clipboard"]')
            ui.write_text(text,'utf-8')
            break
    print(app,item['tag'])
