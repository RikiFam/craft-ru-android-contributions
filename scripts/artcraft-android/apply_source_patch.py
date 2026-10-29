"""Keep desktop accessibility while allowing the native Android activity backend."""
from pathlib import Path

root = Path(__file__).resolve().parents[2]
manifest = root / 'tmp/artcraft/photocraft/repo/crates/ui-egui/Cargo.toml'
text = manifest.read_text(encoding='utf-8')
old = 'eframe = { workspace = true, features = ["wgpu", "accesskit", "clipboard"] }'
new = 'eframe = { workspace = true, features = ["wgpu"] }'
section = '''[target.'cfg(not(target_os = "android"))'.dependencies]
# NativeActivity does not support accesskit; retain it on all existing platforms.
eframe = { workspace = true, features = ["accesskit", "clipboard"] }

'''
if old in text:
    text = text.replace(old, new, 1)
    text = text.replace('[dev-dependencies]', section + '[dev-dependencies]', 1)
    manifest.write_text(text, encoding='utf-8')
else:
    assert new in text and section in text, 'Unexpected UI manifest; review before patching'
print('Android NativeActivity feature separation applied')
