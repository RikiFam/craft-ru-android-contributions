"""Android print hand-off and PDF provider saves, isolated from desktop builds."""
from wire_commands import root, dep, edit

vector = root/'tmp/artcraft/vectorcraft/repo/crates/ui-egui'
dep(vector)
edit(vector/'src/menus/run.rs',
     '"file.print" if p.as_object().is_none_or(|o| o.is_empty()) => crate::dialogs::print::open(app),',
     '''"file.print" if p.as_object().is_none_or(|o| o.is_empty()) => {
            #[cfg(target_os = "android")]
            { crate::print::run(app, &serde_json::json!({"settings": {"copies": 1}})) }
            #[cfg(not(target_os = "android"))]
            { crate::dialogs::print::open(app) }
        },''')

pdf = root/'tmp/artcraft/pdfcraft/repo/crates'
dep(pdf/'print')
dep(pdf/'ui-egui')
spool = pdf/'print/src/spool.rs'
edit(spool, 'parse_lpstat_e, parse_ppd, parse_windows_printers, ppd_text, printers, printers_parsed,',
     'parse_lpstat_e, parse_ppd, parse_windows_printers, ppd_text, printers_parsed,')
text = spool.read_text('utf-8')
if 'pub fn printers()' not in text:
    text += '''
#[cfg(not(target_os = "android"))]
pub use craft_print::printers;
#[cfg(target_os = "android")]
pub fn printers() -> Vec<Printer> {
    vec![Printer { name: "Системное окно Android".into(), default: true, two_sided: None, description: "Выбор принтера и параметров печати".into() }]
}
'''
    spool.write_text(text, 'utf-8')
for name, old in [
    ('submit', 'craft_print::submit(pdf, job).map_err(|e| crate::PrintError::Spool(e.to_string()))'),
    ('submit_with_progress', 'craft_print::submit_with_progress(pdf, job, progress).map_err(|e| crate::PrintError::Spool(e.to_string()))'),
]:
    extra = 'let _ = progress;' if name.endswith('progress') else ''
    edit(spool, '    '+old, '''    #[cfg(target_os = "android")]
    {
        '''+extra+'''
        artcraft_android_platform::print_pdf(pdf, &job.title).map_err(crate::PrintError::Spool)?;
        Ok("Открыто окно печати Android".into())
    }
    #[cfg(not(target_os = "android"))]
    { '''+old+' }')
edit(pdf/'ui-egui/src/editing/save.rs',
     '    write_atomically_with(path, bytes, staging_suffixes())',
     '''    write_atomically_with(path, bytes, staging_suffixes())?;
    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(std::path::Path::new(path)).map_err(std::io::Error::other)?;
    Ok(())''')
deck = root/'tmp/artcraft/deckcraft/repo/crates/ui-egui/src/app/files.rs'
edit(deck, '        let mut params = if params.is_object() { params } else { json!({}) };',
     '''        #[cfg(target_os = "android")]
        {
            match deckcraft_engine::cmd::file::pdf_bytes(&doc, &params) {
                Ok(bytes) => match artcraft_android_platform::print_pdf(&bytes, &title) {
                    Ok(()) => { self.set_status("Открыто окно печати Android"); return true; }
                    Err(error) => { self.set_status(error); return false; }
                },
                Err(error) => { self.set_status(error.to_string()); return false; }
            }
        }
        let mut params = if params.is_object() { params } else { json!({}) };''')

design = root/'tmp/artcraft/designcraft/repo/crates/engine'
dep(design)
design_ui = root/'tmp/artcraft/designcraft/repo/crates/ui-egui'
edit(design_ui/'src/menus/tables.rs', '            "ui:app.printBooklet",',
     '            #[cfg(target_os = "android")]\n            "cmd:file.print",\n            "ui:app.printBooklet",')
edit(design_ui/'src/menus/invoke.rs', '        crate::dialogs::open_print(app);',
     '''        #[cfg(target_os = "android")]
        { let _ = app.run("file.print", json!({})); }
        #[cfg(not(target_os = "android"))]
        crate::dialogs::open_print(app);''')
source = design/'src/cmd/print.rs'
edit(source, '    let printers = craft_print::printers();', '''    #[cfg(target_os = "android")]
    let printers = vec![craft_print::Printer { name: "Системное окно Android".into(), default: true, two_sided: None, description: String::new() }];
    #[cfg(not(target_os = "android"))]
    let printers = craft_print::printers();''')
edit(source, '    #[cfg(not(target_arch = "wasm32"))]\n    print_with(s, p, craft_print::submit)', '''    #[cfg(target_os = "android")]
    return print_with(s, p, |bytes, job| {
        artcraft_android_platform::print_pdf(bytes, &job.title).map_err(craft_print::Error::Spool)?;
        Ok("Открыто окно печати Android".into())
    });
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    print_with(s, p, craft_print::submit)''')
edit(source, '    let command = craft_print::command_preview(&job).map_err(|e| bad("file.print", e.to_string()))?;', '''    #[cfg(target_os = "android")]
    let command = vec!["Android PrintManager".to_string()];
    #[cfg(not(target_os = "android"))]
    let command = craft_print::command_preview(&job).map_err(|e| bad("file.print", e.to_string()))?;''')

cad = root/'tmp/artcraft/cadcraft/repo/crates/ui-egui/src/managers.rs'
edit(cad, '        let mut params = plot_params(&st, None);', '''        let mut params = plot_params(&st, None);
        #[cfg(target_os = "android")]
        if ext == "pdf" {
            let result = app.run("plot", params.clone()).and_then(|value| {
                let encoded = value.get("data").and_then(Value::as_str).ok_or("Не получен PDF для печати")?;
                let bytes = cadcraft_engine::cmd::file::base64_decode(encoded).ok_or("Повреждены данные PDF")?;
                artcraft_android_platform::print_pdf(&bytes, "CADCraft")
            });
            match result {
                Ok(()) => { *open = false; app.session.echo("Открыто окно печати Android"); }
                Err(error) => app.session.echo(error),
            }
            return action;
        }''')

light = root/'tmp/artcraft/lightcraft/repo/crates/ui-egui'
dep(light)
edit(light/'src/menus/run.rs', '''            let mut params = p.clone();
            if p.get("path").is_none() {
                let req = PickRequest::save("Print to PDF", "PDF", &["pdf"], "Print.pdf");''', '''            let mut params = p.clone();
            #[cfg(target_os = "android")]
            if p.get("path").is_none() {
                params["path"] = json!("Android-print.pdf");
                params["androidPrint"] = json!(true);
                return Some(crate::export_task::start_print(app, &params));
            }
            if p.get("path").is_none() {
                let req = PickRequest::save("Print to PDF", "PDF", &["pdf"], "Print.pdf");''')
edit(light/'src/export_task.rs', '''    use lightcraft_engine::print::{compose, output};''', '''    use lightcraft_engine::print::{compose, output};
    #[cfg(target_os = "android")]
    let android_print = params.get("androidPrint").and_then(Value::as_bool).unwrap_or(false);''')
edit(light/'src/export_task.rs', '''                let bytes = output::write(&pages, &settings.job, &title)?;
                guard.check''', '''                let bytes = output::write(&pages, &settings.job, &title)?;
                #[cfg(target_os = "android")]
                if android_print {
                    artcraft_android_platform::print_pdf(&bytes, "LightCraft")?;
                    return Ok(vec![json!({"pages": pages.len(), "photos": photos, "bytes": bytes.len(), "print": true, "androidDialog": true})]);
                }
                guard.check''')
edit(light/'src/dialogs.rs', '        Dialog::Print => "Print to PDF…",', '        Dialog::Print => if cfg!(target_os = "android") { "Печать…" } else { "Print to PDF…" },')
