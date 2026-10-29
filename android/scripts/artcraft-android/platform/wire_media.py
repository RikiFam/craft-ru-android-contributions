"""Install Android host callbacks and durable provider writes for media editors."""
from pathlib import Path
import os

root = Path(__file__).resolve().parents[3]
platform = Path(__file__).resolve().parent
hosts = root / 'scripts/artcraft-android/suite'

def edit(path, old, new):
    text = path.read_text('utf-8-sig')
    if new in text:
        return
    assert old in text, (path, old)
    path.write_text(text.replace(old, new, 1), 'utf-8')

def dependency(app):
    engine = root / f'tmp/artcraft/{app}/repo/crates/engine'
    manifest = engine / 'Cargo.toml'
    text = manifest.read_text('utf-8')
    if 'artcraft-android-platform' not in text:
        relative = os.path.relpath(platform, engine).replace('\\', '/')
        text += '\n[target.\'cfg(target_os = "android")\'.dependencies]\nartcraft-android-platform = { path = "' + relative + '" }\n'
        manifest.write_text(text, 'utf-8')
    return engine

edit(hosts/'filmcraft/src/lib.rs',
     '            editor.session.prefs.general.interface_language',
     '''            editor.hooks.run_dialog = Some(Box::new(|request| {
                use ui::native_dialogs::FileDialog::*;
                let (kind, name) = match request {
                    OpenFiles { .. } => ("many", String::new()),
                    OpenFile { .. } | OpenProject | Relink { .. } => ("open", String::new()),
                    SaveProject { name } | SaveAs { name, .. } => ("save", name),
                    Folder { .. } => ("folder", String::new()),
                };
                artcraft_android_platform::pick_paths(kind, &name)
            }));
            editor.session.prefs.general.interface_language''')
edit(hosts/'effectcraft/src/lib.rs',
     '            editor.session.prefs.general.language',
     '''            let repaint = cc.egui_ctx.clone();
            editor.hooks.file_dialog_async = Some(Box::new(move |request, done| {
                use ui::FileDialog::*;
                let (kind, name) = match request {
                    Open { many, .. } => (if many { "many" } else { "open" }, String::new()),
                    OpenProject => ("open", String::new()),
                    Folder => ("folder", String::new()),
                    Save { default, .. } => ("save", default),
                };
                let answer = artcraft_android_platform::pick_paths(kind, &name);
                let repaint = repaint.clone();
                std::thread::spawn(move || {
                    done(answer.recv().unwrap_or_default());
                    repaint.request_repaint();
                });
            }));
            editor.session.prefs.general.language''')

for app, filename, old in [
    ('effectcraft', 'services.rs', 'config::atomic_write(std::path::Path::new(path), data)'),
    ('filmcraft', 'lib.rs', 'filmcraft_format::atomic_write(std::path::Path::new(path), data)'),
    ('figcraft', 'automation/file_access.rs', 'figcraft_format::write_file_atomic(path, bytes)'),
]:
    engine = dependency(app)
    edit(engine/'src'/filename, old, old + '''?;
        #[cfg(target_os = "android")]
        artcraft_android_platform::commit(std::path::Path::new(path)).map_err(std::io::Error::other)?;
        Ok(())''')

edit(root/'tmp/artcraft/vectorcraft/repo/crates/ui-egui/src/picks.rs',
     '    let s = &app.services;\n    match request {',
     '''    let s = &app.services;
    #[cfg(target_os = "android")]
    if s.start_pick.is_some() { return true; }
    match request {''')

edit(root/'tmp/artcraft/filmcraft/repo/crates/engine/src/export_tools.rs',
     '        let r = filmcraft_export::export(&project, seq, &settings, &provider, &prog).map_err(|e| e.to_string());',
     '''        let r = filmcraft_export::export(&project, seq, &settings, &provider, &prog).map_err(|e| e.to_string());
        #[cfg(target_os = "android")]
        let r = r.and_then(|report| {
            for path in std::iter::once(&report.path).chain(report.extra_files.iter()) {
                artcraft_android_platform::commit(std::path::Path::new(path))?;
            }
            Ok(report)
        });''')
