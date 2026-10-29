"""Resume command-based desktop dialogs after Android's asynchronous picker."""
from pathlib import Path
import os

root = Path(__file__).resolve().parents[3]
platform = Path(__file__).resolve().parent
base = root/'scripts/artcraft-android/suite'

def edit(path, old, new):
    text = path.read_text('utf-8-sig')
    if new in text:
        return
    assert old in text, (path, old)
    path.write_text(text.replace(old, new, 1), 'utf-8')

def dep(crate):
    path = crate/'Cargo.toml'
    text = path.read_text('utf-8')
    if 'artcraft-android-platform' not in text:
        relative = os.path.relpath(platform, crate).replace('\\', '/')
        text += '\n[target.\'cfg(target_os = "android")\'.dependencies]\nartcraft-android-platform = { path = "'+relative+'" }\n'
        path.write_text(text, 'utf-8')

for app in ('cadcraft', 'deckcraft', 'designcraft', 'gridcraft', 'soundcraft'):
    repo = root/f'tmp/artcraft/{app}/repo'
    crate = repo/'crates/ui-egui'
    dep(crate)
    source = crate/('src/lib.rs' if app == 'cadcraft' else 'src/app/commands.rs')
    typ = 'Json' if app == 'gridcraft' else 'Value'
    signature = f'pub fn run(&mut self, id: &str, params: {typ}) -> Result<{typ}, String> {{'
    replacement = signature + f'''
        #[cfg(target_os = "android")]
        {{
            let captured = params.clone();
            artcraft_android_platform::command(id, &captured, || self.run_android_inner(id, params))
        }}
        #[cfg(not(target_os = "android"))]
        self.run_android_inner(id, params)
    }}
    fn run_android_inner(&mut self, id: &str, params: {typ}) -> Result<{typ}, String> {{'''
    edit(source, signature, replacement)
    host = base/app/'src/lib.rs'
    edit(host, '        self.editor.logic(ctx);', '''        if let Some((id, params)) = artcraft_android_platform::replay() {
            if let Err(error) = self.editor.run(&id, params) {
                artcraft_android_platform::report(error);
            }
            artcraft_android_platform::finish_replay();
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
        self.editor.logic(ctx);''')

edit(base/'cadcraft/src/lib.rs', 'ui::Services::default()', '''ui::Services {
                pick_open: Some(Box::new(|| artcraft_android_platform::deferred_pick("open", "", "open"))),
                pick_save: Some(Box::new(|name| artcraft_android_platform::deferred_pick("save", name, "saveas"))),
                pick_lisp: Some(Box::new(|_| artcraft_android_platform::deferred_pick("open", "", "appload"))),
                ..Default::default()
            }''')
for app in ('deckcraft', 'designcraft'):
    edit(base/app/'src/lib.rs', '                ui::Services {', '''                ui::Services {
                    pick_open: Some(Box::new(|_| artcraft_android_platform::deferred_pick("open", "", "file.open"))),
                    pick_save: Some(Box::new(|name| artcraft_android_platform::deferred_pick("save", name, "file.saveAs"))),''')
edit(base/'gridcraft/src/lib.rs', 'ui::Services::default()', '''ui::Services {
                pick_open: Some(Box::new(|| artcraft_android_platform::deferred_pick("open", "", "file.open"))),
                pick_save: Some(Box::new(|name| {
                    let name = if std::path::Path::new(name).extension().is_none() { format!("{name}.xlsx") } else { name.to_string() };
                    artcraft_android_platform::deferred_pick("save", &name, "file.saveAs")
                })),
                print: Some(Box::new(|pdf| {
                    if let Err(error) = artcraft_android_platform::print_pdf(pdf, "GridCraft") {
                        artcraft_android_platform::report(error);
                    }
                })),
                ..Default::default()
            }''')
edit(base/'soundcraft/src/lib.rs', 'ui::Services::default()', '''ui::Services {
                pick_open: Some(Box::new(|cmd, _| artcraft_android_platform::deferred_pick("open", "", cmd).map(PathBuf::from))),
                pick_save: Some(Box::new(|cmd, name| artcraft_android_platform::deferred_pick("save", &name.to_string_lossy(), cmd).map(PathBuf::from))),
                ..Default::default()
            }''')

for app in ('cadcraft', 'gridcraft', 'soundcraft'):
    dep(root/f'tmp/artcraft/{app}/repo/crates/engine')
edit(root/'tmp/artcraft/cadcraft/repo/crates/engine/src/atomic_write.rs',
     '    Ok(())\n}\n\n/// Create a fresh',
     '''    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(path).map_err(io::Error::other)?;
    Ok(())
}

/// Create a fresh''')
edit(root/'tmp/artcraft/gridcraft/repo/crates/engine/src/io.rs',
     '    })\n}\n\n#[cfg(target_arch = "wasm32")]\npub fn write_file',
     '''    })?;
    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(std::path::Path::new(path)).map_err(EngineError::Other)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn write_file''')
edit(root/'tmp/artcraft/soundcraft/repo/crates/engine/src/io.rs',
     '    std::fs::write(path, bytes).map_err(|err| EngineError::Io(format!("{path}: {err}")))',
     '''    std::fs::write(path, bytes).map_err(|err| EngineError::Io(format!("{path}: {err}")))?;
    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(std::path::Path::new(path)).map_err(EngineError::Io)?;
    Ok(())''')
edit(root/'tmp/artcraft/soundcraft/repo/crates/engine/src/io.rs',
     '    sync_dir(file.parent().unwrap_or(Path::new("")));\n    Ok(())',
     '''    sync_dir(file.parent().unwrap_or(Path::new("")));
    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(file).map_err(EngineError::Io)?;
    Ok(())''')
