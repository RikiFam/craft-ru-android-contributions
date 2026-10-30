"""Run native regression tests and repository quality gates, keeping every result."""
import os, subprocess, sys, json
from prepare import ROOT
from prepare_tests import TESTS

out=ROOT/'tmp/artcraft/spelling-builds'
out.mkdir(parents=True,exist_ok=True)
env=os.environ.copy()
env.update(CRAFT_FONTS_DIR=str(ROOT/'tmp/artcraft/craft-fonts'),CRAFT_I18N_DIR=str(ROOT/'tmp/artcraft/craft-i18n'),CARGO_BUILD_JOBS='4',CARGO_TARGET_DIR=str(ROOT/'tmp/artcraft/spelling-check-target'))
mode=sys.argv[1]
report_path=out/f'{mode}-results.json'
report=json.loads(report_path.read_text('utf-8')) if report_path.exists() else {}
for app in sys.argv[2:]:
 repo=ROOT/'tmp/artcraft'/app/'repo'
 if mode=='regression':
  crate=TESTS[app][0] if app in TESTS else 'ui-egui'
  commands=[['cargo','test','-p',f'{app}-{crate}','--lib',('bundled_russian_spell_regression' if app in TESTS else 'russian_'), '--','--test-threads=1']]
 elif mode=='ci':commands=[['cargo','xtask','ci']]
 elif mode=='photo':commands=[['cargo','test','-p','photocraft-text'],['cargo','clippy','-p','photocraft-text','--all-targets','--','-D','warnings'],['cargo','xtask','layers'],['cargo','xtask','wasm'],['cargo','xtask','test-corpus']]
 else:raise ValueError(mode)
 for i,command in enumerate(commands):
  name=f'{app}-{mode}-{i}'
  with (out/f'{name}.log').open('w',encoding='utf-8') as log:
   result=subprocess.run(command,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
  report[name]={'command':command,'exit':result.returncode}
  report_path.write_text(json.dumps(report,indent=2),encoding='utf-8')
  print(name,result.returncode,flush=True)
