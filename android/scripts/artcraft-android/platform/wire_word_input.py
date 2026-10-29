from pathlib import Path
p=Path(__file__).resolve().parents[3]/'tmp/artcraft/wordcraft/repo/crates/ui-egui/src/canvas.rs'
s=p.read_text('utf-8')
old='    crate::keys::observe_ime(app, ui.ctx(), ime_geometry, ime_interrupt);'
new='''    crate::keys::observe_ime(app, ui.ctx(), ime_geometry, ime_interrupt);
    #[cfg(target_os = "android")]
    {
        // Keep the document caret for physical typing after toolbar clicks, but only
        // summon the soft keyboard when the user actually taps the document.
        let id = egui::Id::unique("android-document-keyboard");
        let mut wanted = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
        let on_page = resp.contains_pointer() && resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.latest_pos()))
            .is_some_and(|p| app.canvas.page_rects.iter().any(|r| r.contains(p)));
        if ui.input(|i| i.pointer.any_pressed()) && !on_page {
            wanted = false;
        }
        if focus_click && on_page { wanted = true; }
        ui.ctx().data_mut(|d| d.insert_temp(id, wanted));
        if app.canvas.focused && !wanted { ui.ctx().output_mut(|o| o.ime = None); }
    }'''
marker = '    #[cfg(target_os = "android")]\n    {\n        // Keep the document caret'
if marker in s:
    start = s.index(old)
    end = s.index('\n    }', s.index(marker, start)) + len('\n    }')
    updated = s[:start] + new + s[end:]
else:
    assert old in s
    updated = s.replace(old, new, 1)
if updated != s:
    p.write_text(updated, 'utf-8')
