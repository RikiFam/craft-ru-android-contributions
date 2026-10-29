"""Keep desktop accessibility while building NativeActivity without accesskit."""
from pathlib import Path
import sys
root = Path(__file__).resolve().parents[3]
for app in (sys.argv[1:] or ('filmcraft', 'effectcraft', 'pdfcraft')):
    if app not in ('filmcraft', 'effectcraft', 'pdfcraft'):
        raise ValueError(f'Unsupported app: {app}')
    path = root / f'tmp/artcraft/{app}/repo/crates/ui-egui/Cargo.toml'
    text = path.read_text(encoding='utf-8-sig')
    line = next(line for line in text.splitlines() if line.startswith('eframe ='))
    if '"accesskit"' in line:
        text = text.replace(line, line.replace(', "accesskit"', ''))
        text += '\n[target.\'cfg(not(target_os = "android"))\'.dependencies]\neframe = { workspace = true, features = ["accesskit"] }\n'
        path.write_text(text,encoding='utf-8')
