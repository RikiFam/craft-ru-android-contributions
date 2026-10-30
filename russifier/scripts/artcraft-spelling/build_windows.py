"""Build selected local Windows editions, recording every exit status."""
from pathlib import Path
import subprocess, os, sys, json
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'tmp/artcraft/spelling-builds'
OUT.mkdir(exist_ok=True)
env=os.environ.copy()
env.update(CRAFT_FONTS_DIR=str(ROOT/'tmp/artcraft/craft-fonts'),CRAFT_I18N_DIR=str(ROOT/'tmp/artcraft/craft-i18n'),CARGO_TARGET_DIR=str(ROOT/'tmp/artcraft/photocraft/repo/target'),CARGO_BUILD_JOBS='4',CARGO_PROFILE_RELEASE_LTO='off',CARGO_PROFILE_RELEASE_OPT_LEVEL='2',CARGO_PROFILE_RELEASE_CODEGEN_UNITS='16')
report=json.loads((OUT/'windows-results.json').read_text('utf-8')) if (OUT/'windows-results.json').exists() else {}
for app in sys.argv[1:]:
    repo=ROOT/'tmp/artcraft'/app/'repo'
    with (OUT/f'{app}-windows.log').open('w',encoding='utf-8') as log:
        result=subprocess.run(['cargo','build','--release','-p',app],cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
    report[app]=result.returncode
    (OUT/'windows-results.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(app,'build exit',result.returncode,flush=True)
    if result.returncode:sys.exit(result.returncode)
