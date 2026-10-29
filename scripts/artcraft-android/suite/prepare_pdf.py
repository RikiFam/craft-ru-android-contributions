from pathlib import Path
import shutil

base = Path(__file__).resolve().parent
root = base.parents[2]
source = next((Path.home()/'.cargo/registry/src').glob('*/rfd-0.17.2'))
target = root/'tmp/artcraft/android/rfd-android'
if not target.exists():
    shutil.copytree(source, target)
shutil.copyfile(base/'rfd_android.rs', target/'src/android.rs')
lib = target/'src/lib.rs'
text = lib.read_text(encoding='utf-8')
marker = '\n#[cfg(target_os = "android")]\npub mod android;\n'
if marker not in text:
    lib.write_text(text+marker, encoding='utf-8')
repo = root/'tmp/artcraft/pdfcraft/repo/crates/ui-egui'
manifest = repo/'Cargo.toml'
text = manifest.read_text(encoding='utf-8')
dep = 'arboard = { version = "3.6", default-features = false, features = ["image-data"] }'
if dep in text and '[target.\'cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))\'.dependencies]' not in text:
    text = text.replace(dep, '')
    text += '\n[target.\'cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))\'.dependencies]\n'+dep+'\n'
    manifest.write_text(text, encoding='utf-8')
desktop = '#[cfg(not(target_arch = "wasm32"))]'
androidless = '#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]'
for name, anchor in [('create_ui.rs','pub(crate) fn read_clipboard'),('content_ui.rs','if self.system_clipboard {\n                let img = arboard'),('zoom_snap.rs','if self.system_clipboard {\n            let img = arboard'),('image_pick.rs','fn put_file_on_clipboard(&self, name:')]:
    matches=list((repo/'src').rglob(name))
    assert len(matches)==1, (name,matches)
    path=matches[0]
    text=path.read_text(encoding='utf-8')
    start=text.find(anchor)
    at=text.rfind(desktop,0,start)
    # Only change the immediately preceding cfg on the intended function/block.
    if at>=0 and start-at<100:
        text=text[:at]+text[at:].replace(desktop,androidless,1)
    if name=='create_ui.rs' and 'Android clipboard adapter' not in text:
        text += '\n// Android clipboard adapter: image import remains available through files.\n#[cfg(target_os = "android")]\npub(crate) fn read_clipboard() -> Option<Clip> { None }\n'
    if name=='image_pick.rs':
        text=text.replace('#[cfg(target_arch = "wasm32")]\n    fn put_file_on_clipboard', '#[cfg(any(target_arch = "wasm32", target_os = "android"))]\n    fn put_file_on_clipboard')
    path.write_text(text,encoding='utf-8')
