"""Use Android's flock while preserving the desktop std implementation."""
from pathlib import Path

base = Path(__file__).resolve().parent
root = base.parents[2]
path = root / 'tmp/artcraft/lightcraft/repo/crates/catalog/src/lock.rs'
text = path.read_text(encoding='utf-8')
text = text.split('\n// Android NDK sys/file.h')[0].split('\n// Android file locking adapter.')[0]
assert 'match file.try_lock()' in text or 'match portable_try_lock(&file)' in text
text = text.replace('match file.try_lock()', 'match portable_try_lock(&file)')
text = text.replace('let Err(e) = f.unlock()', 'let Err(e) = portable_unlock(f)')
text += '\n' + (base / 'android_file_lock.rs').read_text(encoding='utf-8')
path.write_text(text, encoding='utf-8')
manifest = path.parent.parent / 'Cargo.toml'
text = manifest.read_text(encoding='utf-8')
if 'rustix =' not in text:
    text += '\n[target.\'cfg(target_os = "android")\'.dependencies]\nrustix = { version = "1.1", features = ["fs"] }\n'
    manifest.write_text(text, encoding='utf-8')
