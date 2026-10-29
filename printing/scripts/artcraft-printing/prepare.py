from pathlib import Path
import shutil
root=Path(__file__).resolve().parents[2]
repo=root/'tmp/artcraft/wordcraft/repo'
cache=Path.home()/'.cargo/git/checkouts/craft-libs-d830b991a08c6c3d/92d392c'
vendor=root/'tmp/artcraft/print-vendor'
if not vendor.exists():
 shutil.copytree(cache,vendor,ignore=shutil.ignore_patterns('.git','target'))
wp=vendor/'Cargo.toml'; ws=wp.read_text('utf-8').replace('members = ["crates/*"]','members = ["crates/print"]'); wp.write_text(ws,'utf-8')
p=vendor/'crates/print/src/windows.rs';s=p.read_text('utf-8')
marker='    let mut builder = PrintTicketBuilder::new(device).map_err(spool_error)?;'
assert marker in s
if 'user_driver_ticket(device)' not in s:
 s=s.replace(marker,marker+'\n    builder.merge(user_driver_ticket(device)?).map_err(spool_error)?;',1)
 s+='''

// Read the current user's complete ticket, including vendor quality/media settings.
// Names travel as environment data, never executable PowerShell source.
fn user_driver_ticket(device: &PrinterDevice) -> Result<PrintTicket, Error> {
    use std::os::windows::process::CommandExt;
    let script = r#"$ErrorActionPreference='Stop'; [Console]::OutputEncoding=New-Object System.Text.UTF8Encoding; Add-Type -AssemblyName System.Printing; $s=New-Object System.Printing.LocalPrintServer; try { $q=$s.GetPrintQueue($env:ARTCRAFT_PRINT_QUEUE); try { $t=$q.UserPrintTicket; if($null -eq $t){$t=$q.DefaultPrintTicket}; $r=New-Object IO.StreamReader($t.GetXmlStream()); try { [Console]::Write($r.ReadToEnd()) } finally { $r.Dispose() } } finally { $q.Dispose() } } finally { $s.Dispose() }"#;
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\\\Windows".into());
    let output = std::process::Command::new(std::path::Path::new(&root).join("System32/WindowsPowerShell/v1.0/powershell.exe"))
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("ARTCRAFT_PRINT_QUEUE", device.name()).creation_flags(0x08000000)
        .output().map_err(spool_error)?;
    if !output.status.success() { return Err(spool_error("Не удалось прочитать свойства принтера. Проверьте его подключение и драйвер.")); }
    let xml = String::from_utf8(output.stdout).map_err(spool_error)?;
    if !xml.contains("PrintTicket") || xml.len() > 4 * 1024 * 1024 { return Err(spool_error("Драйвер вернул некорректные настройки печати")); }
    Ok(PrintTicket::from_xml(xml.into_bytes()))
}
'''
 p.write_text(s,'utf-8')
p=repo/'apps/wordcraft/Cargo.toml';s=p.read_text('utf-8');s=s.replace('craft-print = { git = "https://github.com/storytold/craft-libs", rev = "92d392cff132eaf53e13afff77f38b5c11b73aa0" }','craft-print = { path = "../../../../print-vendor/crates/print" }');p.write_text(s,'utf-8')
p=repo/'crates/ui-egui/src/services.rs';s=p.read_text('utf-8');marker='    pub print_native: Option<printing::Submit>,'
if 'pub printer_properties:' not in s:s=s.replace(marker,marker+'\n    /// Opens driver preferences, without submitting a job.\n    pub printer_properties: Option<Box<dyn Fn(&str) -> Result<(), String>>>,')
p.write_text(s,'utf-8')
p=repo/'apps/wordcraft/src/main.rs';s=p.read_text('utf-8');marker='        print_native: Some(print::submit()),'
if 'printer_properties: ' not in s:s=s.replace(marker,marker+'\n        printer_properties: craft_print::HAS_PRINTER_PREFERENCES.then(|| Box::new(|name: &str| craft_print::open_printer_preferences(name).map_err(|e| e.to_string())) as Box<dyn Fn(&str) -> Result<(), String>>),')
p.write_text(s,'utf-8')
p=repo/'crates/ui-egui/src/print_page.rs';s=p.read_text('utf-8');marker='                    app.ui.print.copies = app.printing.settings.copies;'
if 'app.services.printer_properties' not in s:s=s.replace(marker,'''                    if let Some(properties) = &app.services.printer_properties {
                        if ui.add_enabled(!app.printing.settings.printer.is_empty() && !app.printing.busy(), egui::Button::new(tl!("Printer properties…"))).clicked()
                            && let Err(error) = properties(&app.printing.settings.printer)
                        {
                            app.printing.message = Some(crate::printing::Feedback::Failed(error));
                        }
                        ui.label(egui::RichText::new(tl!("Quality and paper type are set in Printer properties. Copies, color and page layout are set here.")).color(t.text_dim));
                    }
'''+marker)
old='''                    ui.label(tl!("Send the document straight to the system print dialog, or save a copy in another format below."));'''
if 'Select printer and settings…' not in s:s=s.replace(old,'''                    #[cfg(target_os = "android")]
                    {
                        ui.heading(tl!("Printer"));
                        if ui.add(egui::Button::new(tl!("Select printer and settings…")).min_size(vec2(320.0, 44.0))).clicked() {
                            let params = app.ui.print.params();
                            let _ = app.run("ui.print", params);
                        }
                        ui.label(tl!("Select Epson in the next window. Expand the options for paper, color and copies; Other options opens Epson quality settings. Printing starts only after confirmation there."));
                    }
                    #[cfg(not(target_os = "android"))]
'''+old)
p.write_text(s,'utf-8')
labels={
'Auto orientation':'Автоматически',
'Scale to paper size':'Подогнать под размер бумаги',
'No Scaling':'Без масштабирования',
'Portrait':'Книжная',
'Landscape':'Альбомная',
'Printer properties…':'Свойства принтера…',
'Quality and paper type are set in Printer properties. Copies, color and page layout are set here.':'Качество и тип бумаги — в свойствах принтера. Число копий, цвет и расположение страниц — здесь.',
'Select printer and settings…':'Выбрать принтер и настроить печать…',
'Select Epson in the next window. Expand the options for paper, color and copies; Other options opens Epson quality settings. Printing starts only after confirmation there.':'В следующем окне выберите Epson. Разверните параметры бумаги, цвета и копий. Кнопка «Другие параметры» открывает настройки качества Epson. Печать начнётся только после подтверждения в том окне.'}
p=root/'tmp/artcraft/craft-i18n/locales/ru/wordcraft.tsv';s=p.read_text('utf-8')
for key,value in labels.items():
 if key+'\t' not in s:s+='\n\t'+key+'\t'+value
p.write_text(s.rstrip()+'\n','utf-8')
(root/'scripts/artcraft-printing/ru.tsv').write_text('\n'.join('\t'+k+'\t'+v for k,v in labels.items())+'\n','utf-8')

p=vendor/"crates/print/src/windows.rs"; s=p.read_text("utf-8")
if "mod user_preferences_tests" not in s:
 p.write_text(s+(root/"scripts/artcraft-printing/driver-test.rs").read_text("utf-8-sig"),"utf-8")
