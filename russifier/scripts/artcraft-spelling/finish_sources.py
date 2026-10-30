"""Format changed Rust files and record bundled-asset attribution."""
from prepare import ROOT, HERE, BACKUP
from prepare_panels import READERS
from prepare_tests import TESTS
import subprocess

apps=list(TESTS)+list(READERS)
for app in apps:
 repo=ROOT/'tmp/artcraft'/app/'repo'
 files=set()
 backup=BACKUP/app
 if backup.exists():
  files.update(repo/p.relative_to(backup) for p in backup.rglob('*.rs'))
 for crate in ['ui-egui','engine','text']:
  for name in ['russian_spell.rs','proof_panel.rs']:
   p=repo/'crates'/crate/'src'/name
   if p.exists():
    files.add(p)
    files.add(p.with_name('lib.rs'))
 if app in TESTS:
  crate,path,_=TESTS[app]; files.add(repo/'crates'/crate/'src'/path)
 for p in files:
  subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',str(p)],cwd=repo,check=True)
 attribution=repo/'ATTRIBUTION.md'
 if not attribution.exists():attribution=repo/'ASSETS.md'
 if not attribution.exists():attribution=repo/'ATTRIBUTION.md'
 text=attribution.read_text('utf-8') if attribution.exists() else '# Asset attribution\n'
 if 'Local Russian spelling dictionary' not in text:
  text+='\n## Local Russian spelling dictionary\n\nBundled from the build input `scripts/artcraft-russian/dictionaries/ru_RU.aff` and `ru_RU.dic` in the local ArtCraft workspace. Author: Alexander I. Lebedev; changes by Laszlo Nemeth. Source: https://github.com/LibreOffice/dictionaries/tree/master/ru_RU . Permissive redistribution license; full notice in `RUSSIAN-SPELLING-NOTICE.txt`. The affix parser is adapted from DeckCraft under MIT OR Apache-2.0.\n'
  attribution.write_text(text,encoding='utf-8')
 (repo/'RUSSIAN-SPELLING-NOTICE.txt').write_text((HERE/'NOTICE').read_text('utf-8'),encoding='utf-8')
print('Formatted spelling changes and recorded attribution for',len(apps),'editors')
